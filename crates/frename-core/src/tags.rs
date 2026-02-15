//! Tag management for file classification.

/// A single tag that can be applied to a file.
#[derive(Debug, Clone)]
pub struct Tag {
    /// The tag text.
    tag: String,
    /// Whether this tag is currently checked.
    checked: bool,
}

impl Tag {
    /// Create a new tag with the given text.
    pub fn new(tag: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            checked: false,
        }
    }

    /// Get the tag text.
    pub fn tag(&self) -> &str {
        &self.tag
    }

    /// Whether this tag is checked.
    pub fn is_checked(&self) -> bool {
        self.checked
    }

    /// Set the checked state of this tag.
    pub fn set_checked(&mut self, checked: bool) {
        self.checked = checked;
    }

    /// Toggle the checked state of this tag.
    pub fn toggle(&mut self) {
        self.checked = !self.checked;
    }
}

/// A collection of available tags.
#[derive(Clone, Debug)]
pub struct TagList {
    tags: Vec<Tag>,
}

impl TagList {
    /// Create a new TagList with the default set of hardcoded tags.
    pub fn new() -> Self {
        Self {
            tags: default_tags(),
        }
    }

    /// Get the list of all available tags.
    pub fn tags(&self) -> &[Tag] {
        &self.tags
    }

    /// Get a mutable reference to the list of all available tags.
    pub fn tags_mut(&mut self) -> &mut [Tag] {
        &mut self.tags
    }
}

impl Default for TagList {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns the default set of 100 hardcoded tags.
fn default_tags() -> Vec<Tag> {
    let names = [
        "Action",
        "Adventure",
        "Animation",
        "Architecture",
        "Art",
        "Astronomy",
        "Biography",
        "Blog",
        "Business",
        "Celebration",
        "Classic",
        "Comedy",
        "Concert",
        "Cooking",
        "Dance",
        "Design",
        "Documentary",
        "Drama",
        "Education",
        "Entertainment",
        "Environment",
        "Event",
        "Experimental",
        "Family",
        "Fantasy",
        "Fashion",
        "Finance",
        "Fitness",
        "Food",
        "Gaming",
        "Gardening",
        "Geography",
        "Health",
        "History",
        "Holiday",
        "Home Improvement",
        "Horror",
        "How-To",
        "Humor",
        "Indie",
        "Industrial",
        "Interview",
        "Journalism",
        "Kids",
        "Landscape",
        "Language",
        "Lecture",
        "Lifestyle",
        "Literature",
        "Live Stream",
        "Mathematics",
        "Medicine",
        "Military",
        "Motivation",
        "Music",
        "Mystery",
        "Mythology",
        "Nature",
        "News",
        "Outdoors",
        "Parody",
        "Performance",
        "Pets",
        "Philosophy",
        "Photography",
        "Physics",
        "Podcast",
        "Politics",
        "Portrait",
        "Presentation",
        "Psychology",
        "Puzzle",
        "Reality",
        "Religion",
        "Retro",
        "Review",
        "Romance",
        "Satire",
        "Science",
        "Science Fiction",
        "Short Film",
        "Social Media",
        "Space",
        "Sports",
        "Suspense",
        "Technology",
        "Thriller",
        "Time-Lapse",
        "Travel",
        "Tutorial",
        "Underwater",
        "Urban",
        "Vlog",
        "Weather",
        "Wedding",
        "Western",
        "Wildlife",
        "Workout",
        "Workshop",
        "Yoga",
    ];

    names.iter().map(|name| Tag::new(*name)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_creation() {
        let tag = Tag::new("Action");
        assert_eq!(tag.tag(), "Action");
    }

    #[test]
    fn test_tag_list_has_100_tags() {
        let list = TagList::new();
        assert_eq!(list.tags().len(), 100);
    }

    #[test]
    fn test_tag_list_first_and_last() {
        let list = TagList::new();
        assert_eq!(list.tags().first().unwrap().tag(), "Action");
        assert_eq!(list.tags().last().unwrap().tag(), "Yoga");
    }
}
