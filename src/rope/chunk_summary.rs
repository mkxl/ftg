use derive_more::{Constructor, Into};
use zed_sum_tree::ContextLessSummary;

// NOTE: [Clone] required for [ContextLessSummary]
#[derive(Clone, Constructor, Copy, Default)]
pub struct ChunkSummary {
    num_extended_graphemes: usize,
    num_newlines: usize,
}

impl ContextLessSummary for ChunkSummary {
    fn zero() -> Self {
        Self::default()
    }

    fn add_summary(&mut self, summary: &Self) {
        self.num_extended_graphemes += summary.num_extended_graphemes;
        self.num_newlines += summary.num_newlines;
    }
}

macro_rules! impl_dimension {
    ($summary_type:ty, $dimension_type:ty, $dimension_field:ident) => {
        impl<'a> ::zed_sum_tree::Dimension<'a, $summary_type> for $dimension_type {
            fn zero(_context: <$summary_type as ::zed_sum_tree::Summary>::Context<'a>) -> Self {
                Self::default()
            }

            fn add_summary(
                &mut self,
                summary: &'a $summary_type,
                _context: <$summary_type as ::zed_sum_tree::Summary>::Context<'a>,
            ) {
                self.0 += summary.$dimension_field;
            }
        }
    };
}

// NOTE: [Clone] required for [Dimension]
#[derive(Constructor, Clone, Default, Into)]
pub struct LineOffset(usize);

// NOTE: [Clone] required for [Dimension]
#[derive(Constructor, Clone, Default, Into)]
pub struct ExtendedGraphemeOffset(usize);

impl_dimension!(ChunkSummary, LineOffset, num_newlines);
impl_dimension!(ChunkSummary, ExtendedGraphemeOffset, num_newlines);
