//! Shared pagination types for list endpoints.

use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

/// Maximum allowed page size.
pub const MAX_PAGE_SIZE: usize = 100;

/// Default page size.
pub const DEFAULT_PAGE_SIZE: usize = 50;

/// Pagination query parameters for list endpoints.
#[derive(Debug, Clone, Deserialize, IntoParams)]
pub struct PaginationParams {
    /// Maximum number of items to return (default: 50, max: 100).
    #[serde(default = "default_limit")]
    #[param(minimum = 1, maximum = 100, default = 50)]
    pub limit: usize,

    /// Number of items to skip before returning results.
    #[serde(default)]
    #[param(minimum = 0, default = 0)]
    pub offset: usize,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            limit: DEFAULT_PAGE_SIZE,
            offset: 0,
        }
    }
}

impl PaginationParams {
    /// Get the effective limit, clamped to MAX_PAGE_SIZE.
    pub fn effective_limit(&self) -> usize {
        self.limit.min(MAX_PAGE_SIZE).max(1)
    }

    /// Apply pagination to a slice, returning (paginated_items, total).
    pub fn paginate<T: Clone>(&self, items: &[T]) -> (Vec<T>, usize) {
        let total = items.len();
        let limit = self.effective_limit();
        let offset = self.offset.min(total);
        let end = (offset + limit).min(total);
        let paginated = items[offset..end].to_vec();
        (paginated, total)
    }
}

fn default_limit() -> usize {
    DEFAULT_PAGE_SIZE
}

/// Paginated response wrapper.
///
/// All list endpoints should use this format for consistent pagination.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PaginatedResponse<T: ToSchema> {
    /// The items in this page.
    pub items: Vec<T>,
    /// Total number of items across all pages.
    pub total: usize,
    /// Maximum items per page (as requested).
    pub limit: usize,
    /// Offset from the start of the collection.
    pub offset: usize,
}

impl<T: ToSchema> PaginatedResponse<T> {
    /// Create a new paginated response from pagination params and total count.
    pub fn new(items: Vec<T>, total: usize, params: &PaginationParams) -> Self {
        Self {
            items,
            total,
            limit: params.effective_limit(),
            offset: params.offset,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_pagination() {
        let params = PaginationParams::default();
        assert_eq!(params.limit, DEFAULT_PAGE_SIZE);
        assert_eq!(params.offset, 0);
    }

    #[test]
    fn test_effective_limit_clamped() {
        let params = PaginationParams {
            limit: 500,
            offset: 0,
        };
        assert_eq!(params.effective_limit(), MAX_PAGE_SIZE);
    }

    #[test]
    fn test_effective_limit_minimum() {
        let params = PaginationParams {
            limit: 0,
            offset: 0,
        };
        assert_eq!(params.effective_limit(), 1);
    }

    #[test]
    fn test_paginate_basic() {
        let items: Vec<i32> = (0..20).collect();
        let params = PaginationParams {
            limit: 5,
            offset: 0,
        };
        let (result, total) = params.paginate(&items);
        assert_eq!(total, 20);
        assert_eq!(result, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_paginate_with_offset() {
        let items: Vec<i32> = (0..20).collect();
        let params = PaginationParams {
            limit: 5,
            offset: 10,
        };
        let (result, total) = params.paginate(&items);
        assert_eq!(total, 20);
        assert_eq!(result, vec![10, 11, 12, 13, 14]);
    }

    #[test]
    fn test_paginate_offset_beyond_end() {
        let items: Vec<i32> = (0..5).collect();
        let params = PaginationParams {
            limit: 10,
            offset: 100,
        };
        let (result, total) = params.paginate(&items);
        assert_eq!(total, 5);
        assert!(result.is_empty());
    }

    #[test]
    fn test_paginate_empty() {
        let items: Vec<i32> = vec![];
        let params = PaginationParams::default();
        let (result, total) = params.paginate(&items);
        assert_eq!(total, 0);
        assert!(result.is_empty());
    }
}
