use std::iter::Iterator;
use crate::util::CrdtCollection;

#[derive(Clone)]
pub struct SmallVectorImpl<V> {
    next_local_id: u32,
    local_author: u16,
    spans: im::Vector<Span>,
    document: V,
}

impl<V: CrdtCollection> SmallVectorImpl<V> {
    pub fn new(local_author: u16) -> Self {
        Self::with_data(V::new(), local_author)
    }

    pub fn with_data(data: V, local_author: u16) -> Self {
        let mut spans = im::Vector::new();
        // Insert a dummy span since we insert always after another element, this is the only
        // possible way to insert at the first position.
        spans.insert(
            0,
            Span {
                document_index: 0,
                length: data.length() as u32 + 1,
                start_id: 0,
                author: 0,
                deleted: false,
            },
        );
        Self {
            next_local_id: 0,
            local_author,
            spans,
            document: data,
        }
    }

    pub fn next_local_id(&mut self) -> u32 {
        let id = self.next_local_id;
        self.next_local_id += 1;
        id
    }

    pub fn author(&self) -> u16 {
        self.local_author
    }

    #[inline(always)]
    pub fn position(&self, index: usize) -> LocalId {
        let index = index as u32 + 1;
        let (span_id, &span) = self.spans.iter()
            .enumerate()
            .find(|span|!span.1.deleted && span.1.document_index <= index && span.1.document_index + span.1.length > index)
            .unwrap();
        
        LocalId {
            span_index: span_id,
            offset: index - span.document_index,
            span
        }
    }

    #[inline(always)]
    pub fn stable_position(&self, id: StableId) -> LocalId {
        let (span_id, &span) = self.spans.iter()
            .enumerate()
            .find(|span|span.1.author == id.author && span.1.start_id <= id.id && span.1.start_id + span.1.length > id.id)
            .unwrap();

        LocalId {
            span_index: span_id,
            offset: if !span.deleted { id.id - span.start_id } else { 0 },
            span
        }
    }

    #[inline(always)]
    pub fn insert(&mut self, previous: LocalId, current: StableId, element: V::Element) {
        let mut span_id = previous.span_index;
        let span = previous.span;
        let previous_id = previous.span.start_id + previous.offset;

        if span.deleted {
            // The span is already deleted: It has no character in the document
            self.spans.insert(
                span_id + 1,
                Span {
                    document_index: span.document_index,
                    length: 1,
                    start_id: current.id,
                    author: current.author,
                    deleted: false,
                },
            );
            self.document
                .insert(span.document_index as usize - 1, element);
        } else {
            let document_index = previous_id - span.start_id + span.document_index + 1;
            self.document.insert(document_index as usize - 1, element);

            if span.start_id + span.length == current.id
                && previous_id + 1 == current.id
                && span.author == current.author
            {
                // We are behind the last item of this span and are the same author: Extend the span
                // This is the case we optimise for, since most of the time typing happens continuously.
                self.spans[span_id].length += 1;
            } else {
                // We are inside the span of another author or jumped back, therefore we need to
                // create a new span and split the span if necessary.

                let new_length = previous_id - span.start_id + 1;

                // Insert the new span after the old span
                // span_id
                self.spans.insert(
                    span_id + 1,
                    Span {
                        document_index,
                        length: 1,
                        start_id: current.id,
                        author: current.author,
                        deleted: false,
                    },
                );

                if new_length < span.length {
                    // Split off the rest:
                    self.spans.insert(
                        span_id + 2,
                        Span {
                            document_index: document_index + 1, // Insert after the current character
                            length: span.length - new_length,   // The remaining length
                            start_id: span.start_id + new_length, // The id after the last id inside the old span
                            author: span.author,
                            deleted: false,
                        },
                    );

                    // Reduce the length
                    // new length will always be <= span.length therefore this is the only case
                    // in which we need to change something
                    self.spans[span_id].length = new_length;

                    span_id += 1;
                }

                span_id += 1;
            }
        }

        for span in self.spans.iter_mut().skip(span_id + 1) {
            // advance each following span.
            span.document_index += 1;
        }
    }


    #[inline(always)]
    pub fn delete(&mut self, local_id: LocalId) {
        let span_index = local_id.span_index;
        let span = local_id.span;
        let mut next_span = span_index + 1;
        let id = local_id.span.start_id + local_id.offset;
        let author = local_id.span.author;

        // Otherwise someone else already deleted this id.
        if !span.deleted {
            self.document
                .remove((id - span.start_id + span.document_index) as usize - 1);

            let mut new_length = span.length - 1;

            let other = if self.spans.len() > span_index + 1 {
                let next_span = &mut self.spans[span_index + 1];

                if next_span.start_id == id + 1
                    && next_span.author == author
                    && next_span.deleted
                    && span.start_id + span.length == id + 1
                    && span.author == author
                {
                    // prepend
                    next_span.document_index -= 1;
                    next_span.start_id -= 1;
                    next_span.length += 1;
                    false
                } else {
                    true
                }
            } else {
                true
            };

            if other {
                new_length = id - span.start_id;

                // The span with the newly deleted item
                self.spans.insert(
                    span_index + 1,
                    Span {
                        document_index: span.document_index + new_length,
                        length: 1,
                        start_id: id,
                        author,
                        deleted: true,
                    },
                );

                if new_length < span.length - 1 {
                    // The additional items of the original span behind the deleted item
                    self.spans.insert(
                        span_index + 2,
                        Span {
                            document_index: span.document_index + new_length,
                            length: span.length - new_length - 1,
                            start_id: id + 1,
                            author,
                            deleted: false,
                        },
                    );

                    next_span += 1;
                }

                next_span += 1;
            }

            for span in self.spans.iter_mut().skip(next_span) {
                span.document_index -= 1;
            }

            if new_length == 0 {
                // The span we are inside of has only one character: remove the span!
                self.spans.remove(span_index);

                // Merge if possible
                let next_span = self.spans[span_index];

                // Since we have a dummy item at position 0 span_id will never be 0
                let previous = &mut self.spans[span_index - 1];

                if previous.start_id + previous.length == next_span.start_id
                    && previous.author == next_span.author
                {
                    previous.length += next_span.length;
                    self.spans.remove(span_index);
                }
            } else {
                let span_ref = &mut self.spans[span_index];
                span_ref.length = new_length;
            }
        }
    }

    //TODO: decide whether we should move this into crate::cmrdt::SmallVector
    // pro: it is the only place where we need it
    // contra: we would need to export Span also and this struct would become a leaky abstraction
    pub fn merge(&mut self, _other: &Self) {
        unimplemented!()
    }

    /// returns the underlying document. Since this CRDT does not keep a history, document contains
    /// the accurate list.
    pub fn document(&self) -> &V {
        &self.document
    }

    pub fn dbg_spans(&self) {
        println!("Spans: {:#?}", self.spans);
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct LocalId {
    span_index: usize,
    offset: u32,
    span: Span,
}

impl LocalId {
    #[inline(always)]
    pub fn stable(self) -> StableId {
        StableId {
            author: self.span.author,
            id: self.span.start_id + self.offset,
        }
    }

    pub fn id(self) -> u32 {
        self.span.start_id + self.offset
    }

    #[inline(always)]
    pub fn position(self) -> usize {
        if !self.span.deleted {
            (self.span.document_index + self.offset) as usize + 1
        } else {
            self.span.document_index as usize + 1
        }
    }

    pub fn author(self) -> u16 {
        self.span.author
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct StableId {
    author: u16,
    id: u32,
}

impl StableId {
    pub fn new(author: u16, id: u32) -> Self {
        Self {
            author,
            id,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Span {
    pub document_index: u32,
    pub length: u32,
    pub start_id: u32,
    pub author: u16,
    pub deleted: bool,
}
