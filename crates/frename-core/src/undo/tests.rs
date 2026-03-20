/// Unit tests for the undo system (derived from UNDO_TESTS.md).
///
/// These tests cover History management and ReorderTagCommand.
/// NavigateFileCommand is tested in integration (would require real fs rename).
#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::SystemTime;

    use crate::db::fake_app_storage::FakeAppStorage;
    use crate::{Directory, File, FileId, FileSnapshot, StoredTag, TagList};
    use crate::undo::{
        History, NavigateFileCommand, ReorderTagCommand, ToggleTagCommand, PasteTagsCommand,
        DeleteTagCommand, CreateTagCommand, SaveTagCommand, StarTagCommand, UndoContext,
    };
    use uuid::Uuid;

    // --- helpers ---

    fn make_store_with_tags(names: &[&str]) -> FakeAppStorage {
        names.iter().enumerate().fold(FakeAppStorage::new(), |store, (i, name)| {
            store.add_stored_tag(StoredTag::with_all(Uuid::new_v4(), *name, (i + 1) as i64, false), 0)
        })
    }

    fn snapshot_with_tags(names: &[&str]) -> FileSnapshot {
        FileSnapshot::new(
            names.iter().map(|s| s.to_string()).collect(),
            "file",
            "mp4",
            "file.mp4",
        )
    }

    fn empty_directory() -> Directory<FakeAppStorage> {
        Directory::with_files(PathBuf::from("C:/test"), vec![], FakeAppStorage::new())
    }

    // --- History management ---

    #[test]
    fn history_empty_at_startup() {
        let history = History::<FakeAppStorage, FakeAppStorage>::new(50);
        assert!(!history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn can_undo_redo_update_through_cycle() {
        let store = make_store_with_tags(&["A", "B"]);
        let snapshot = snapshot_with_tags(&["A", "B"]);
        let mut tag_list = TagList::new(store, snapshot);
        let mut dir = empty_directory();
        let mut history = History::new(50);

        let id_a = tag_list.checked_tag_id_at(0).unwrap();
        let fi = tag_list.checked_index_of(id_a).unwrap();
        tag_list.reorder_tag_to_index(id_a, 1);
        history.push(Box::new(ReorderTagCommand { moved_id: id_a, from_index: fi, to_index: 1 }));

        assert!(history.can_undo());
        assert!(!history.can_redo());

        let mut ctx = UndoContext { directory: &mut dir, tag_list: &mut tag_list };
        history.undo(&mut ctx).unwrap();
        assert!(!history.can_undo());
        assert!(history.can_redo());

        let mut ctx = UndoContext { directory: &mut dir, tag_list: &mut tag_list };
        history.redo(&mut ctx).unwrap();
        assert!(history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn push_after_undo_clears_redo_stack() {
        let store = make_store_with_tags(&["A", "B", "C"]);
        let snapshot = snapshot_with_tags(&["A", "B", "C"]);
        let mut tag_list = TagList::new(store, snapshot);
        let mut dir = empty_directory();
        let mut history = History::new(50);

        // Two pushes
        let id_a = tag_list.checked_tag_id_at(0).unwrap();
        let id_b = tag_list.checked_tag_id_at(1).unwrap();

        tag_list.reorder_tag_to_index(id_a, 2);
        history.push(Box::new(ReorderTagCommand { moved_id: id_a, from_index: 0, to_index: 2 }));
        tag_list.reorder_tag_to_index(id_b, 1);
        history.push(Box::new(ReorderTagCommand { moved_id: id_b, from_index: 0, to_index: 1 }));

        // Undo one → redo stack has 1
        {
            let mut ctx = UndoContext { directory: &mut dir, tag_list: &mut tag_list };
            history.undo(&mut ctx).unwrap();
        }
        assert!(history.can_redo());

        // Push new action → redo stack cleared
        history.push(Box::new(ReorderTagCommand { moved_id: id_a, from_index: 0, to_index: 1 }));
        assert!(!history.can_redo());
        assert!(history.can_undo());
    }

    #[test]
    fn history_respects_max_depth() {
        let store = make_store_with_tags(&["A", "B"]);
        let snapshot = snapshot_with_tags(&["A", "B"]);
        let mut tag_list = TagList::new(store, snapshot);
        let mut dir = empty_directory();
        let mut history = History::new(3);

        let id_a = tag_list.checked_tag_id_at(0).unwrap();

        // Push 3 commands (fills max)
        for _ in 0..3 {
            history.push(Box::new(ReorderTagCommand { moved_id: id_a, from_index: 0, to_index: 0 }));
        }
        // Push one more — should drop the oldest
        history.push(Box::new(ReorderTagCommand { moved_id: id_a, from_index: 0, to_index: 0 }));

        // Still exactly max_depth entries: undo 3 times and then can_undo should be false
        let mut count = 0;
        while history.can_undo() {
            let mut ctx = UndoContext { directory: &mut dir, tag_list: &mut tag_list };
            history.undo(&mut ctx).unwrap();
            count += 1;
        }
        assert_eq!(count, 3);
    }

    #[test]
    fn undo_with_empty_stack_is_noop() {
        let store = FakeAppStorage::new();
        let mut tag_list = TagList::new(store, FileSnapshot::default());
        let mut dir = empty_directory();
        let mut history = History::<FakeAppStorage, FakeAppStorage>::new(50);

        let mut ctx = UndoContext { directory: &mut dir, tag_list: &mut tag_list };
        assert!(history.undo(&mut ctx).is_ok());
        assert!(!history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn redo_with_empty_stack_is_noop() {
        let store = FakeAppStorage::new();
        let mut tag_list = TagList::new(store, FileSnapshot::default());
        let mut dir = empty_directory();
        let mut history = History::<FakeAppStorage, FakeAppStorage>::new(50);

        let mut ctx = UndoContext { directory: &mut dir, tag_list: &mut tag_list };
        assert!(history.redo(&mut ctx).is_ok());
    }

    // --- ReorderTagCommand ---

    #[test]
    fn undo_reorder_tag_restores_original_order() {
        let store = make_store_with_tags(&["Action", "Comedy", "Summer"]);
        let snapshot = snapshot_with_tags(&["Action", "Comedy", "Summer"]);
        let mut tag_list = TagList::new(store, snapshot);

        let id_action = tag_list.checked_tag_id_at(0).unwrap();
        let fi = tag_list.checked_index_of(id_action).unwrap();
        assert_eq!(fi, 0);

        tag_list.reorder_tag_to_index(id_action, 2);
        assert_eq!(tag_list.file_snapshot().tags(), ["Comedy", "Summer", "Action"]);

        let mut history = History::new(50);
        history.push(Box::new(ReorderTagCommand { moved_id: id_action, from_index: 0, to_index: 2 }));

        let mut dir = empty_directory();
        {
            let mut ctx = UndoContext { directory: &mut dir, tag_list: &mut tag_list };
            history.undo(&mut ctx).unwrap();
        }
        assert_eq!(tag_list.file_snapshot().tags(), ["Action", "Comedy", "Summer"]);
        assert!(!history.can_undo());
        assert!(history.can_redo());
    }

    #[test]
    fn redo_reorder_tag_reapplies_move() {
        let store = make_store_with_tags(&["Action", "Comedy", "Summer"]);
        let snapshot = snapshot_with_tags(&["Action", "Comedy", "Summer"]);
        let mut tag_list = TagList::new(store, snapshot);

        let id_action = tag_list.checked_tag_id_at(0).unwrap();
        tag_list.reorder_tag_to_index(id_action, 2);

        let mut history = History::new(50);
        history.push(Box::new(ReorderTagCommand { moved_id: id_action, from_index: 0, to_index: 2 }));

        let mut dir = empty_directory();
        {
            let mut ctx = UndoContext { directory: &mut dir, tag_list: &mut tag_list };
            history.undo(&mut ctx).unwrap();
        }
        assert_eq!(tag_list.file_snapshot().tags(), ["Action", "Comedy", "Summer"]);

        {
            let mut ctx = UndoContext { directory: &mut dir, tag_list: &mut tag_list };
            history.redo(&mut ctx).unwrap();
        }
        assert_eq!(tag_list.file_snapshot().tags(), ["Comedy", "Summer", "Action"]);
        assert!(history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn undo_backward_reorder_restores_order() {
        let store = make_store_with_tags(&["Action", "Comedy", "Summer"]);
        let snapshot = snapshot_with_tags(&["Action", "Comedy", "Summer"]);
        let mut tag_list = TagList::new(store, snapshot);

        let id_summer = tag_list.checked_tag_id_at(2).unwrap();
        tag_list.reorder_tag_to_index(id_summer, 0);
        assert_eq!(tag_list.file_snapshot().tags(), ["Summer", "Action", "Comedy"]);

        let mut history = History::new(50);
        history.push(Box::new(ReorderTagCommand { moved_id: id_summer, from_index: 2, to_index: 0 }));

        let mut dir = empty_directory();
        let mut ctx = UndoContext { directory: &mut dir, tag_list: &mut tag_list };
        history.undo(&mut ctx).unwrap();
        assert_eq!(tag_list.file_snapshot().tags(), ["Action", "Comedy", "Summer"]);
    }

    #[test]
    fn multiple_reorders_unwind_in_sequence() {
        let store = make_store_with_tags(&["Action", "Comedy", "Summer"]);
        let snapshot = snapshot_with_tags(&["Action", "Comedy", "Summer"]);
        let mut tag_list = TagList::new(store, snapshot);
        let mut dir = empty_directory();
        let mut history = History::new(50);

        let id_action = tag_list.checked_tag_id_at(0).unwrap();
        let id_summer = tag_list.checked_tag_id_at(2).unwrap(); // before first reorder

        // Move Action 0→2: Comedy, Summer, Action
        tag_list.reorder_tag_to_index(id_action, 2);
        history.push(Box::new(ReorderTagCommand { moved_id: id_action, from_index: 0, to_index: 2 }));

        // In new order: Comedy(0), Summer(1), Action(2). Move Summer (now at 1) to 0: Summer, Comedy, Action
        // We need the new id_summer position
        let id_summer_new = tag_list.checked_tag_id_at(1).unwrap();
        assert_eq!(id_summer_new, id_summer);
        tag_list.reorder_tag_to_index(id_summer, 0);
        history.push(Box::new(ReorderTagCommand { moved_id: id_summer, from_index: 1, to_index: 0 }));

        assert_eq!(tag_list.file_snapshot().tags(), ["Summer", "Comedy", "Action"]);

        // Undo second: back to Comedy, Summer, Action
        {
            let mut ctx = UndoContext { directory: &mut dir, tag_list: &mut tag_list };
            history.undo(&mut ctx).unwrap();
        }
        assert_eq!(tag_list.file_snapshot().tags(), ["Comedy", "Summer", "Action"]);

        // Undo first: back to Action, Comedy, Summer
        {
            let mut ctx = UndoContext { directory: &mut dir, tag_list: &mut tag_list };
            history.undo(&mut ctx).unwrap();
        }
        assert_eq!(tag_list.file_snapshot().tags(), ["Action", "Comedy", "Summer"]);
        assert!(!history.can_undo());
        assert!(history.can_redo());
    }

    #[test]
    fn undo_reorder_fails_when_tag_deleted() {
        let store = make_store_with_tags(&["Action", "Comedy"]);
        let snapshot = snapshot_with_tags(&["Action", "Comedy"]);
        let mut tag_list = TagList::new(store, snapshot);

        let id_action = tag_list.checked_tag_id_at(0).unwrap();
        tag_list.reorder_tag_to_index(id_action, 1);

        let mut history = History::new(50);
        history.push(Box::new(ReorderTagCommand { moved_id: id_action, from_index: 0, to_index: 1 }));

        // Delete the tag from the list (simulate external deletion)
        tag_list.remove_stored_tag_by_id(id_action).unwrap();

        let mut dir = empty_directory();
        let mut ctx = UndoContext { directory: &mut dir, tag_list: &mut tag_list };
        let result = history.undo(&mut ctx);
        assert!(result.is_err());
        // Stack unchanged — still can_undo
        assert!(history.can_undo());
        assert!(!history.can_redo());
    }

    // --- NavigateFileCommand (pure logic, no real fs rename) ---

    #[test]
    fn navigate_command_undo_restores_selection() {
        let dir_path = PathBuf::from("C:/test");
        let files = vec![
            File::from_path(dir_path.join("a.mp4"), SystemTime::UNIX_EPOCH),
            File::from_path(dir_path.join("b.mp4"), SystemTime::UNIX_EPOCH),
        ];
        let file_a_id = files[0].id();
        let file_b_id = files[1].id();
        let mut directory = Directory::with_files(&dir_path, files, FakeAppStorage::new());
        directory.select_index(1); // start at index 1

        let store = FakeAppStorage::new();
        let mut tag_list = TagList::new(store, FileSnapshot::default());

        let snap_before = FileSnapshot::new(vec![], "a", "mp4", "a.mp4");
        let snap_after = snap_before.clone();

        let mut history = History::new(50);
        history.push(Box::new(NavigateFileCommand {
            file_id: file_a_id,
            to_file_id: file_b_id,
            path_before: dir_path.join("a.mp4"),
            path_after: dir_path.join("a.mp4"), // same path (no rename)
            snapshot_before: snap_before.clone(),
            snapshot_after: snap_after.clone(),
        }));

        {
            let mut ctx = UndoContext { directory: &mut directory, tag_list: &mut tag_list };
            history.undo(&mut ctx).unwrap();
        }
        assert_eq!(directory.selected_index(), Some(0));
    }

    #[test]
    fn navigate_command_redo_re_applies_selection() {
        let dir_path = PathBuf::from("C:/test");
        let files = vec![
            File::from_path(dir_path.join("a.mp4"), SystemTime::UNIX_EPOCH),
            File::from_path(dir_path.join("b.mp4"), SystemTime::UNIX_EPOCH),
        ];
        let file_a_id = files[0].id();
        let file_b_id = files[1].id();
        let mut directory = Directory::with_files(&dir_path, files, FakeAppStorage::new());
        directory.select_index(1);

        let store = FakeAppStorage::new();
        let mut tag_list = TagList::new(store, FileSnapshot::default());

        let mut history = History::new(50);
        history.push(Box::new(NavigateFileCommand {
            file_id: file_a_id,
            to_file_id: file_b_id,
            path_before: dir_path.join("a.mp4"),
            path_after: dir_path.join("a.mp4"),
            snapshot_before: FileSnapshot::new(vec![], "a", "mp4", "a.mp4"),
            snapshot_after: FileSnapshot::new(vec![], "a", "mp4", "a.mp4"),
        }));

        {
            let mut ctx = UndoContext { directory: &mut directory, tag_list: &mut tag_list };
            history.undo(&mut ctx).unwrap();
        }
        assert_eq!(directory.selected_index(), Some(0));

        {
            let mut ctx = UndoContext { directory: &mut directory, tag_list: &mut tag_list };
            history.redo(&mut ctx).unwrap();
        }
        assert_eq!(directory.selected_index(), Some(1));
    }

    // --- ToggleTagCommand ---

    #[test]
    fn undo_toggle_tag_restores_unchecked() {
        let store = make_store_with_tags(&["Action"]);
        let snapshot = snapshot_with_tags(&["Action"]); // Action is checked
        let mut tag_list = TagList::new(store, snapshot);
        let mut dir = empty_directory();

        let id = tag_list.checked_tag_id_at(0).unwrap();
        let was_checked = true; // it was checked before toggle
        tag_list.toggle_by_id(id); // now unchecked

        let mut history = History::new(50);
        history.push(Box::new(ToggleTagCommand { tag_id: id, was_checked }));

        {
            let mut ctx = UndoContext { directory: &mut dir, tag_list: &mut tag_list };
            history.undo(&mut ctx).unwrap();
        }
        // Should be checked again
        assert!(tag_list.get_tag(id).unwrap().is_checked());
    }

    #[test]
    fn redo_toggle_tag_reapplies_uncheck() {
        let store = make_store_with_tags(&["Action"]);
        let snapshot = snapshot_with_tags(&["Action"]);
        let mut tag_list = TagList::new(store, snapshot);
        let mut dir = empty_directory();

        let id = tag_list.checked_tag_id_at(0).unwrap();
        tag_list.toggle_by_id(id); // now unchecked

        let mut history = History::new(50);
        history.push(Box::new(ToggleTagCommand { tag_id: id, was_checked: true }));

        {
            let mut ctx = UndoContext { directory: &mut dir, tag_list: &mut tag_list };
            history.undo(&mut ctx).unwrap();
        }
        assert!(tag_list.get_tag(id).unwrap().is_checked());

        {
            let mut ctx = UndoContext { directory: &mut dir, tag_list: &mut tag_list };
            history.redo(&mut ctx).unwrap();
        }
        assert!(!tag_list.get_tag(id).unwrap().is_checked());
    }

    // --- PasteTagsCommand ---

    #[test]
    fn undo_paste_tags_restores_before_snapshot() {
        let store = make_store_with_tags(&["Action", "Comedy"]);
        let snap_before = snapshot_with_tags(&["Action"]);
        let snap_after = snapshot_with_tags(&["Action", "Comedy"]);
        let mut tag_list = TagList::new(store, snap_after.clone());
        let mut dir = empty_directory();

        let mut history = History::new(50);
        history.push(Box::new(PasteTagsCommand {
            snapshot_before: snap_before.clone(),
            snapshot_after: snap_after.clone(),
        }));

        {
            let mut ctx = UndoContext { directory: &mut dir, tag_list: &mut tag_list };
            history.undo(&mut ctx).unwrap();
        }
        assert_eq!(tag_list.file_snapshot().tags(), ["Action"]);
    }

    #[test]
    fn redo_paste_tags_reapplies_after_snapshot() {
        let store = make_store_with_tags(&["Action", "Comedy"]);
        let snap_before = snapshot_with_tags(&["Action"]);
        let snap_after = snapshot_with_tags(&["Action", "Comedy"]);
        let mut tag_list = TagList::new(store, snap_after.clone());
        let mut dir = empty_directory();

        let mut history = History::new(50);
        history.push(Box::new(PasteTagsCommand {
            snapshot_before: snap_before.clone(),
            snapshot_after: snap_after.clone(),
        }));

        {
            let mut ctx = UndoContext { directory: &mut dir, tag_list: &mut tag_list };
            history.undo(&mut ctx).unwrap();
        }
        assert_eq!(tag_list.file_snapshot().tags(), ["Action"]);

        {
            let mut ctx = UndoContext { directory: &mut dir, tag_list: &mut tag_list };
            history.redo(&mut ctx).unwrap();
        }
        assert_eq!(tag_list.file_snapshot().tags(), ["Action", "Comedy"]);
    }

    // --- DeleteTagCommand ---

    #[test]
    fn undo_delete_tag_restores_tag() {
        let store = make_store_with_tags(&["Action", "Comedy"]);
        let snapshot = snapshot_with_tags(&["Action"]);
        let mut tag_list = TagList::new(store, snapshot);
        let mut dir = empty_directory();

        let id = tag_list.checked_tag_id_at(0).unwrap();
        // Capture before deleting
        let (name, color_index, was_stored, was_starred, was_checked, sort_order) =
            tag_list.capture_delete_data(id).unwrap();
        tag_list.remove_stored_tag_by_id(id).unwrap();
        assert!(tag_list.get_tag(id).is_none());

        let mut history = History::new(50);
        history.push(Box::new(DeleteTagCommand {
            tag_id: id, tag_name: name, color_index, was_stored, was_starred, was_checked, sort_order,
        }));

        {
            let mut ctx = UndoContext { directory: &mut dir, tag_list: &mut tag_list };
            history.undo(&mut ctx).unwrap();
        }
        assert!(tag_list.get_tag(id).is_some());
    }

    #[test]
    fn redo_delete_tag_removes_tag_again() {
        let store = make_store_with_tags(&["Action"]);
        let snapshot = snapshot_with_tags(&["Action"]);
        let mut tag_list = TagList::new(store, snapshot);
        let mut dir = empty_directory();

        let id = tag_list.checked_tag_id_at(0).unwrap();
        let (name, color_index, was_stored, was_starred, was_checked, sort_order) =
            tag_list.capture_delete_data(id).unwrap();
        tag_list.remove_stored_tag_by_id(id).unwrap();

        let mut history = History::new(50);
        history.push(Box::new(DeleteTagCommand {
            tag_id: id, tag_name: name, color_index, was_stored, was_starred, was_checked, sort_order,
        }));

        {
            let mut ctx = UndoContext { directory: &mut dir, tag_list: &mut tag_list };
            history.undo(&mut ctx).unwrap();
        }
        assert!(tag_list.get_tag(id).is_some());

        {
            let mut ctx = UndoContext { directory: &mut dir, tag_list: &mut tag_list };
            history.redo(&mut ctx).unwrap();
        }
        assert!(tag_list.get_tag(id).is_none());
    }

    // --- CreateTagCommand ---

    #[test]
    fn undo_create_tag_removes_tag() {
        let store = FakeAppStorage::new();
        let mut tag_list = TagList::new(store, FileSnapshot::default());
        let mut dir = empty_directory();

        let id = tag_list.create_new_tag("NewTag").unwrap();
        tag_list.save_tag(id).unwrap();

        let mut history = History::new(50);
        history.push(Box::new(CreateTagCommand {
            tag_id: id,
            tag_name: "NewTag".to_string(),
            color_index: tag_list.get_tag(id).unwrap().color_index(),
        }));

        {
            let mut ctx = UndoContext { directory: &mut dir, tag_list: &mut tag_list };
            history.undo(&mut ctx).unwrap();
        }
        assert!(tag_list.get_tag(id).is_none());
    }

    #[test]
    fn redo_create_tag_recreates_tag() {
        let store = FakeAppStorage::new();
        let mut tag_list = TagList::new(store, FileSnapshot::default());
        let mut dir = empty_directory();

        let id = tag_list.create_new_tag("NewTag").unwrap();
        let color_index = tag_list.get_tag(id).unwrap().color_index();
        tag_list.save_tag(id).unwrap();

        let mut history = History::new(50);
        history.push(Box::new(CreateTagCommand {
            tag_id: id,
            tag_name: "NewTag".to_string(),
            color_index,
        }));

        {
            let mut ctx = UndoContext { directory: &mut dir, tag_list: &mut tag_list };
            history.undo(&mut ctx).unwrap();
        }
        assert!(tag_list.get_tag(id).is_none());

        {
            let mut ctx = UndoContext { directory: &mut dir, tag_list: &mut tag_list };
            history.redo(&mut ctx).unwrap();
        }
        assert!(tag_list.get_tag(id).is_some());
        assert!(tag_list.get_tag(id).unwrap().is_stored());
    }

    // --- SaveTagCommand ---

    #[test]
    fn undo_save_tag_unsaves_it() {
        let store = FakeAppStorage::new();
        let mut tag_list = TagList::new(store, FileSnapshot::default());
        let mut dir = empty_directory();

        let id = tag_list.create_new_tag("NewTag").unwrap();
        assert!(!tag_list.get_tag(id).unwrap().is_stored());

        tag_list.save_tag(id).unwrap();
        let color_index = tag_list.get_tag(id).unwrap().color_index();
        assert!(tag_list.get_tag(id).unwrap().is_stored());

        let mut history = History::new(50);
        history.push(Box::new(SaveTagCommand { tag_id: id, color_index }));

        {
            let mut ctx = UndoContext { directory: &mut dir, tag_list: &mut tag_list };
            history.undo(&mut ctx).unwrap();
        }
        assert!(!tag_list.get_tag(id).unwrap().is_stored());
    }

    #[test]
    fn redo_save_tag_saves_it_again() {
        let store = FakeAppStorage::new();
        let mut tag_list = TagList::new(store, FileSnapshot::default());
        let mut dir = empty_directory();

        let id = tag_list.create_new_tag("NewTag").unwrap();
        tag_list.save_tag(id).unwrap();
        let color_index = tag_list.get_tag(id).unwrap().color_index();

        let mut history = History::new(50);
        history.push(Box::new(SaveTagCommand { tag_id: id, color_index }));

        {
            let mut ctx = UndoContext { directory: &mut dir, tag_list: &mut tag_list };
            history.undo(&mut ctx).unwrap();
        }
        assert!(!tag_list.get_tag(id).unwrap().is_stored());

        {
            let mut ctx = UndoContext { directory: &mut dir, tag_list: &mut tag_list };
            history.redo(&mut ctx).unwrap();
        }
        assert!(tag_list.get_tag(id).unwrap().is_stored());
    }

    // --- StarTagCommand ---

    #[test]
    fn undo_star_tag_unstars_it() {
        let store = make_store_with_tags(&["Action"]);
        let snapshot = snapshot_with_tags(&[]);
        let mut tag_list = TagList::new(store, snapshot);
        let mut dir = empty_directory();

        // Find the stored tag id (it's in the display list but not checked)
        let id = *tag_list.filtered_display_tag_ids().first().unwrap();
        assert!(!tag_list.get_tag(id).unwrap().is_starred());

        tag_list.star_tag(id).unwrap();
        assert!(tag_list.get_tag(id).unwrap().is_starred());

        let mut history = History::new(50);
        history.push(Box::new(StarTagCommand { tag_id: id, was_starred: false }));

        {
            let mut ctx = UndoContext { directory: &mut dir, tag_list: &mut tag_list };
            history.undo(&mut ctx).unwrap();
        }
        assert!(!tag_list.get_tag(id).unwrap().is_starred());
    }

    #[test]
    fn redo_star_tag_stars_it_again() {
        let store = make_store_with_tags(&["Action"]);
        let snapshot = snapshot_with_tags(&[]);
        let mut tag_list = TagList::new(store, snapshot);
        let mut dir = empty_directory();

        let id = *tag_list.filtered_display_tag_ids().first().unwrap();
        tag_list.star_tag(id).unwrap();

        let mut history = History::new(50);
        history.push(Box::new(StarTagCommand { tag_id: id, was_starred: false }));

        {
            let mut ctx = UndoContext { directory: &mut dir, tag_list: &mut tag_list };
            history.undo(&mut ctx).unwrap();
        }
        assert!(!tag_list.get_tag(id).unwrap().is_starred());

        {
            let mut ctx = UndoContext { directory: &mut dir, tag_list: &mut tag_list };
            history.redo(&mut ctx).unwrap();
        }
        assert!(tag_list.get_tag(id).unwrap().is_starred());
    }

    #[test]
    fn navigate_command_undo_fails_gracefully_when_file_not_found() {
        let dir_path = PathBuf::from("C:/test");
        let files = vec![
            File::from_path(dir_path.join("a.mp4"), SystemTime::UNIX_EPOCH),
        ];
        let file_a_id = files[0].id();
        let mut directory = Directory::with_files(&dir_path, files, FakeAppStorage::new());
        directory.select_index(0);

        let store = FakeAppStorage::new();
        let mut tag_list = TagList::new(store, FileSnapshot::default());

        let mut history = History::new(50);
        // Use a bogus to_file_id that doesn't exist in the directory (redo would fail)
        // For undo: file_id must exist so the rename reverts, but then select_by_id(file_id) is used.
        // Test that redo fails when to_file_id is bogus.
        let bogus_id = FileId::new();
        history.push(Box::new(NavigateFileCommand {
            file_id: file_a_id,
            to_file_id: bogus_id,
            path_before: dir_path.join("a.mp4"),
            path_after: dir_path.join("a.mp4"),
            snapshot_before: FileSnapshot::new(vec![], "a", "mp4", "a.mp4"),
            snapshot_after: FileSnapshot::new(vec![], "a", "mp4", "a.mp4"),
        }));

        {
            let mut ctx = UndoContext { directory: &mut directory, tag_list: &mut tag_list };
            history.undo(&mut ctx).unwrap(); // undo selects file_a_id — succeeds
        }
        // redo should fail: to_file_id is bogus
        let mut ctx = UndoContext { directory: &mut directory, tag_list: &mut tag_list };
        let result = history.redo(&mut ctx);
        assert!(result.is_err());
        // Stack unchanged after error
        assert!(history.can_redo());
    }
}
