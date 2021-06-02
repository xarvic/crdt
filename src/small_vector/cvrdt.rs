use crate::small_vector::structure::{CrdtCollection, Span};
use crate::CvRDT;
use std::iter::Iterator;

#[derive(Clone)]
pub struct Vector<V> {
    next_local_id: u64,
    local_author: u16,
    spans: im::Vector<Span>,
    document: V,
}

impl<V: CrdtCollection> Vector<V> {
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

    pub(crate) fn span_index(&self, author: u16, id: u32) -> usize {
        self.spans
            .iter()
            .position(|span| {
                span.author == author && span.start_id <= id && span.start_id + span.length > id
            })
            .expect("cant find span!")
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

#[derive(Clone)]
pub enum VectorUpdate<T> {
    Insert {
        previous_author: u16,
        previous_id: u32,
        this_author: u16,
        this_id: u32,
        element: T,
    },
    Delete {
        author: u16,
        id: u32,
    },
}

impl<V: CrdtCollection> CvRDT for Vector<V> {
    type Update = VectorUpdate<V::Element>;

    fn update(&mut self, update: Self::Update) {
        match update {
            VectorUpdate::Insert {
                previous_author,
                previous_id,
                this_author,
                this_id,
                element,
            } => {
                let mut span_id = self.span_index(previous_author, previous_id);
                let span = self.spans[span_id];
                if span.deleted {
                    // The span is already deleted: It has no character in the document
                    self.spans.insert(
                        span_id + 1,
                        Span {
                            document_index: span.document_index,
                            length: 1,
                            start_id: this_id,
                            author: this_author,
                            deleted: false,
                        },
                    );
                    self.document
                        .insert(span.document_index as usize - 1, element);
                } else {
                    let document_index = previous_id - span.start_id + span.document_index + 1;
                    self.document.insert(document_index as usize - 1, element);

                    if span.start_id + span.length == this_id
                        && previous_id + 1 == this_id
                        && span.author == this_author
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
                                start_id: this_id,
                                author: this_author,
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
            VectorUpdate::Delete { author, id } => {
                let span_id = self.span_index(author, id);
                let span = self.spans[span_id];
                let mut shift_span = span_id + 1;

                // Otherwise someone else already deleted this id.
                if !span.deleted {
                    self.document
                        .remove((id - span.start_id + span.document_index) as usize - 1);

                    let mut new_length = span.length - 1;

                    let other = if self.spans.len() > span_id + 1 {
                        let next_span = &mut self.spans[span_id + 1];

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
                            span_id + 1,
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
                                span_id + 2,
                                Span {
                                    document_index: span.document_index + new_length,
                                    length: span.length - new_length - 1,
                                    start_id: id + 1,
                                    author,
                                    deleted: false,
                                },
                            );

                            shift_span += 1;
                        }

                        shift_span += 1;
                    }

                    for span in self.spans.iter_mut().skip(shift_span) {
                        span.document_index -= 1;
                    }

                    if new_length == 0 {
                        // The span we are inside of has only one character: remove the span!
                        self.spans.remove(span_id);

                        // Merge if possible
                        let next_span = self.spans[span_id];

                        // Since we have a dummy item at position 0 span_id will never be 0
                        let previous = &mut self.spans[span_id - 1];

                        if previous.start_id + previous.length == next_span.start_id
                            && previous.author == next_span.author
                        {
                            previous.length += next_span.length;
                            self.spans.remove(span_id);
                        }
                    } else {
                        let span_ref = &mut self.spans[span_id];
                        span_ref.length = new_length;
                    }
                }
            }
        }
    }
}
