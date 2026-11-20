//! Platform-neutral catalog filter selection transitions.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterAction {
    Category,
    Favorites,
    All,
}

pub fn filter_selection(action: FilterAction, category: Option<&str>) -> (Option<&str>, bool) {
    match action {
        FilterAction::Category => (category, false),
        FilterAction::Favorites => (None, true),
        FilterAction::All => (None, false),
    }
}

#[cfg(test)]
mod tests {
    use super::{filter_selection, FilterAction};

    #[test]
    fn category_favorites_and_all_reset_selection_consistently() {
        assert_eq!(
            filter_selection(FilterAction::Category, Some("天气")),
            (Some("天气"), false)
        );
        assert_eq!(
            filter_selection(FilterAction::Category, None),
            (None, false)
        );
        assert_eq!(
            filter_selection(FilterAction::Favorites, Some("ignored")),
            (None, true)
        );
        assert_eq!(
            filter_selection(FilterAction::All, Some("ignored")),
            (None, false)
        );
    }
}
