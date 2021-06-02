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
        let index = index as u32;
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
    pub fn insert(&mut self, previous: LocalId, id: StableId, value: V::Element) {
        let mut next_span = previous.span_index + 1;
        if previous.span.deleted {
            // The span is already deleted: It has no character in the document

            // Insert a new span after it
            self.spans.insert(
                previous.span_index + 1,
                Span {
                    document_index: previous.span.document_index,
                    length: 1,
                    start_id: id.id,
                    author: id.author,
                    deleted: false,
                },
            );

            // Insert the characrter where the previous span started
            self.document.insert(previous.span.document_index as usize - 1, value);

            next_span += 1;
        } else {

            // The span is not deleted, therefore we insert the character after the position of
            // previous id.
            let document_index = previous.position() + 1;
            self.document.insert(document_index- 1, value);

            if previous.span_end_id() == id.id
                && previous.id() + 1 == id.id
                && previous.span.author == id.author
            {
                // We are behind the last item of this span and are the same author: Extend the span
                // This is the case we optimise for, since most of the time typing happens continuously.
                self.spans[previous.span_index()].length += 1;
            } else {

                // We are inside the span of another author or jumped back, therefore we need to
                // create a new span and split the span if necessary.
                let new_length = previous.offset + 1;

                // Insert the new span after the old span
                // span_id
                self.spans.insert(
                    previous.span_index + 1,
                    Span {
                        document_index: document_index as u32,
                        length: 1,
                        start_id: id.id,
                        author: id.author,
                        deleted: false,
                    },
                );

                if new_length < previous.span.length {
                    // Split off the rest:
                    self.spans.insert(
                        previous.span_index + 2,
                        Span {
                            document_index: document_index as u32 + 1,    // Insert after the current character
                            length: previous.span.length - new_length,    // The remaining length
                            start_id: previous.span_start_id() + new_length, // The id after the last id inside the old span
                            author: previous.span.author,
                            deleted: false,
                        },
                    );

                    // Reduce the length
                    // new length will always be <= span.length therefore this is the only case
                    // in which we need to change something
                    self.spans[previous.span_index].length = new_length;

                    next_span += 1;
                }

                next_span += 1;
            }
        }

        for span in self.spans.iter_mut().skip(next_span) {
            // advance each following span.
            span.document_index += 1;
        }
    }


    #[inline(always)]
    pub fn delete(&mut self, local_id: LocalId) {
        let mut next_span = local_id.span_index() + 1;

        // Otherwise someone else already deleted this id.
        if !local_id.span.deleted {
            self.document
                .remove(local_id.position() as usize - 1);

            let mut new_length = local_id.span_length() - 1;

            let other = if self.spans.len() > local_id.span_index() + 1 {
                let next_span = &mut self.spans[local_id.span_index() + 1];

                if next_span.start_id == local_id.id() + 1
                    && next_span.author == local_id.author()
                    && next_span.deleted
                    && local_id.span_length() == local_id.offset() + 1
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
                new_length = local_id.offset;

                // The span with the newly deleted item
                self.spans.insert(
                    local_id.span_index() + 1,
                    Span {
                        document_index: local_id.span_start_id() + new_length,
                        length: 1,
                        start_id: local_id.id(),
                        author: local_id.author(),
                        deleted: true,
                    },
                );

                if new_length < local_id.span.length - 1 {
                    // The additional items of the original span behind the deleted item
                    self.spans.insert(
                        local_id.span_index() + 2,
                        Span {
                            document_index: local_id.span_start_position() + new_length,
                            length: local_id.span_length() - new_length - 1,
                            start_id: local_id.id() + 1,
                            author: local_id.author(),
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
                self.spans.remove(local_id.span_index());

                // Merge if possible
                let next_span = self.spans[local_id.span_index()];

                // Since we have a dummy item at position 0 span_id will never be 0
                let previous = &mut self.spans[local_id.span_index() - 1];

                if previous.start_id + previous.length == next_span.start_id
                    && previous.author == next_span.author
                {
                    previous.length += next_span.length;
                    self.spans.remove(local_id.span_index());
                }
            } else {
                let span_ref = &mut self.spans[local_id.span_index()];
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
            (self.span.document_index + self.offset) as usize
        } else {
            0
        }
    }

    pub fn author(self) -> u16 {
        self.span.author
    }

    pub fn span_length(self) -> u32 {
        self.span.length
    }

    #[inline(always)]
    pub fn span_start_id(self) -> u32 {
        self.span.start_id
    }

    #[inline(always)]
    pub fn span_end_id(self) -> u32 {
        self.span.start_id + self.span.length
    }

    #[inline(always)]
    pub fn span_start_position(self) -> u32 {
        self.span.document_index
    }

    #[inline(always)]
    pub fn span_end_position(self) -> u32 {
        self.span.document_index + self.span.length
    }

    pub fn span_index(self) -> usize {
        self.span_index
    }

    pub fn offset(self) -> u32 {
        self.offset
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
