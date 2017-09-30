use xrl::{Line, Operation, OperationType, Update};

// use errors::*;

#[derive(Clone, Debug)]
pub struct LineCache {
    pub invalid_before: u64,
    pub lines: Vec<Line>,
    pub invalid_after: u64,
}

impl LineCache {
    pub fn new() -> Self {
        LineCache {
            invalid_before: 0,
            lines: vec![],
            invalid_after: 0,
        }
    }

    pub fn update(&mut self, update: Update) {
        let LineCache {
            ref mut lines,
            ref mut invalid_before,
            ref mut invalid_after,
        } = *self;
        let helper = UpdateHelper {
            old_lines: lines,
            invalid_before: invalid_before,
            invalid_after: invalid_after,
            new_lines: Vec::new(),
        };
        helper.update(update.operations);
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}

struct UpdateHelper<'a, 'b, 'c> {
    old_lines: &'a mut Vec<Line>,
    invalid_before: &'b mut u64,
    new_lines: Vec<Line>,
    invalid_after: &'c mut u64,
}

impl<'a, 'b, 'c> UpdateHelper<'a, 'b, 'c> {
    fn get_fields_mut(&mut self) -> (&mut Vec<Line>, &mut Vec<Line>) {
        let UpdateHelper {
            ref mut old_lines,
            ref mut new_lines,
            ..
        } = *self;
        (old_lines, new_lines)
    }

    fn apply_copy(&mut self, nb_lines: u64) {
        info!("copying {} lines", nb_lines);
        let (old_lines, new_lines) = self.get_fields_mut();
        new_lines.extend(old_lines.drain(0..nb_lines as usize))
    }

    fn apply_skip(&mut self, nb_lines: u64) {
        info!("skipping {} lines", nb_lines);
        let _ = self.old_lines.drain(0..nb_lines as usize).last();
    }

    fn apply_invalidate(&mut self, nb_lines: u64) {
        info!("invalidating {} lines", nb_lines);
        if self.new_lines.is_empty() {
            *self.invalid_before = nb_lines;
        } else {
            *self.invalid_after = nb_lines;
        }
    }

    fn apply_insert(&mut self, mut lines: Vec<Line>) {
        info!("inserting {} lines", lines.len());
        self.new_lines.extend(lines.drain(..).map(|mut line| {
            trim_new_line(&mut line.text);
            line
        }));
    }

    fn apply_update(&mut self, nb_lines: u64, lines: Vec<Line>) {
        info!("updating {} lines", nb_lines);
        let (old_lines, new_lines) = self.get_fields_mut();
        new_lines.extend(
            old_lines
                .drain(0..nb_lines as usize)
                .zip(lines.into_iter())
                .map(|(mut old_line, update)| {
                    old_line.cursor = update.cursor;
                    old_line.styles = update.styles;
                    old_line
                }),
        )
    }

    fn update(mut self, operations: Vec<Operation>) {
        *self.invalid_before = 0;
        *self.invalid_after = 0;
        for op in operations {
            match op.operation_type {
                OperationType::Copy_ => (&mut self).apply_copy(op.nb_lines),
                OperationType::Skip => (&mut self).apply_skip(op.nb_lines),
                OperationType::Invalidate => (&mut self).apply_invalidate(op.nb_lines),
                OperationType::Insert => (&mut self).apply_insert(op.lines),
                OperationType::Update => (&mut self).apply_update(op.nb_lines, op.lines),
            }
        }
        *self.old_lines = self.new_lines;
    }
}

fn trim_new_line(text: &mut String) {
    if let Some('\n') = text.chars().last() {
        text.pop();
    }
}
