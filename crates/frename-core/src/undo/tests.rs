#![allow(clippy::module_inception)]
/// Unit tests for the undo system (derived from UNDO_TESTS.md).
///
/// These tests cover History management and ReorderTagCommand.
/// NavigateFileCommand is tested in integration (would require real fs rename).
#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::SystemTime;

    use crate::db::fake_app_storage::FakeAppStorage;
    use crate::undo::{
        AddMarkerCommand, CheckTagsCommand, CreateTagCommand, DeleteMarkerCommand,
        DeleteTagCommand, History, NavigateFileCommand, PasteTagsCommand, RenameFileCommand,
        ReorderTagCommand, SaveTagCommand, SetCommentCommand, SetMarkerColorCommand,
        SetMarkerNameCommand, SetMarkerSpanCommand, SetMarkerTextCommand, SetSegmentCommand,
        StarTagCommand, SyncTagOrderCommand, ToggleTagCommand, UndoContext,
    };
    use crate::{Directory, File, FileId, FileSnapshot, Marker, MarkerColor, StoredTag, TagList};
    use uuid::Uuid;

    // --- helpers ---

    fn make_store_with_tags(names: &[&str]) -> FakeAppStorage {
        names
            .iter()
            .enumerate()
            .fold(FakeAppStorage::new(), |store, (i, name)| {
                store.add_stored_tag(
                    StoredTag::with_all(Uuid::new_v4(), *name, (i + 1) as i64, false),
                    0,
                )
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
        history.push(Box::new(ReorderTagCommand {
            moved_id: id_a,
            from_index: fi,
            to_index: 1,
        }));

        assert!(history.can_undo());
        assert!(!history.can_redo());

        let mut ctx = UndoContext {
            directory: &mut dir,
            tag_list: &mut tag_list,
        };
        history.undo(&mut ctx).unwrap();
        assert!(!history.can_undo());
        assert!(history.can_redo());

        let mut ctx = UndoContext {
            directory: &mut dir,
            tag_list: &mut tag_list,
        };
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
        history.push(Box::new(ReorderTagCommand {
            moved_id: id_a,
            from_index: 0,
            to_index: 2,
        }));
        tag_list.reorder_tag_to_index(id_b, 1);
        history.push(Box::new(ReorderTagCommand {
            moved_id: id_b,
            from_index: 0,
            to_index: 1,
        }));

        // Undo one → redo stack has 1
        {
            let mut ctx = UndoContext {
                directory: &mut dir,
                tag_list: &mut tag_list,
            };
            history.undo(&mut ctx).unwrap();
        }
        assert!(history.can_redo());

        // Push new action → redo stack cleared
        history.push(Box::new(ReorderTagCommand {
            moved_id: id_a,
            from_index: 0,
            to_index: 1,
        }));
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
            history.push(Box::new(ReorderTagCommand {
                moved_id: id_a,
                from_index: 0,
                to_index: 0,
            }));
        }
        // Push one more — should drop the oldest
        history.push(Box::new(ReorderTagCommand {
            moved_id: id_a,
            from_index: 0,
            to_index: 0,
        }));

        // Still exactly max_depth entries: undo 3 times and then can_undo should be false
        let mut count = 0;
        while history.can_undo() {
            let mut ctx = UndoContext {
                directory: &mut dir,
                tag_list: &mut tag_list,
            };
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

        let mut ctx = UndoContext {
            directory: &mut dir,
            tag_list: &mut tag_list,
        };
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

        let mut ctx = UndoContext {
            directory: &mut dir,
            tag_list: &mut tag_list,
        };
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
        assert_eq!(
            tag_list.file_snapshot().tags(),
            ["Comedy", "Summer", "Action"]
        );

        let mut history = History::new(50);
        history.push(Box::new(ReorderTagCommand {
            moved_id: id_action,
            from_index: 0,
            to_index: 2,
        }));

        let mut dir = empty_directory();
        {
            let mut ctx = UndoContext {
                directory: &mut dir,
                tag_list: &mut tag_list,
            };
            history.undo(&mut ctx).unwrap();
        }
        assert_eq!(
            tag_list.file_snapshot().tags(),
            ["Action", "Comedy", "Summer"]
        );
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
        history.push(Box::new(ReorderTagCommand {
            moved_id: id_action,
            from_index: 0,
            to_index: 2,
        }));

        let mut dir = empty_directory();
        {
            let mut ctx = UndoContext {
                directory: &mut dir,
                tag_list: &mut tag_list,
            };
            history.undo(&mut ctx).unwrap();
        }
        assert_eq!(
            tag_list.file_snapshot().tags(),
            ["Action", "Comedy", "Summer"]
        );

        {
            let mut ctx = UndoContext {
                directory: &mut dir,
                tag_list: &mut tag_list,
            };
            history.redo(&mut ctx).unwrap();
        }
        assert_eq!(
            tag_list.file_snapshot().tags(),
            ["Comedy", "Summer", "Action"]
        );
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
        assert_eq!(
            tag_list.file_snapshot().tags(),
            ["Summer", "Action", "Comedy"]
        );

        let mut history = History::new(50);
        history.push(Box::new(ReorderTagCommand {
            moved_id: id_summer,
            from_index: 2,
            to_index: 0,
        }));

        let mut dir = empty_directory();
        let mut ctx = UndoContext {
            directory: &mut dir,
            tag_list: &mut tag_list,
        };
        history.undo(&mut ctx).unwrap();
        assert_eq!(
            tag_list.file_snapshot().tags(),
            ["Action", "Comedy", "Summer"]
        );
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
        history.push(Box::new(ReorderTagCommand {
            moved_id: id_action,
            from_index: 0,
            to_index: 2,
        }));

        // In new order: Comedy(0), Summer(1), Action(2). Move Summer (now at 1) to 0: Summer, Comedy, Action
        // We need the new id_summer position
        let id_summer_new = tag_list.checked_tag_id_at(1).unwrap();
        assert_eq!(id_summer_new, id_summer);
        tag_list.reorder_tag_to_index(id_summer, 0);
        history.push(Box::new(ReorderTagCommand {
            moved_id: id_summer,
            from_index: 1,
            to_index: 0,
        }));

        assert_eq!(
            tag_list.file_snapshot().tags(),
            ["Summer", "Comedy", "Action"]
        );

        // Undo second: back to Comedy, Summer, Action
        {
            let mut ctx = UndoContext {
                directory: &mut dir,
                tag_list: &mut tag_list,
            };
            history.undo(&mut ctx).unwrap();
        }
        assert_eq!(
            tag_list.file_snapshot().tags(),
            ["Comedy", "Summer", "Action"]
        );

        // Undo first: back to Action, Comedy, Summer
        {
            let mut ctx = UndoContext {
                directory: &mut dir,
                tag_list: &mut tag_list,
            };
            history.undo(&mut ctx).unwrap();
        }
        assert_eq!(
            tag_list.file_snapshot().tags(),
            ["Action", "Comedy", "Summer"]
        );
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
        history.push(Box::new(ReorderTagCommand {
            moved_id: id_action,
            from_index: 0,
            to_index: 1,
        }));

        // Delete the tag from the list (simulate external deletion)
        tag_list.remove_stored_tag_by_id(id_action).unwrap();

        let mut dir = empty_directory();
        let mut ctx = UndoContext {
            directory: &mut dir,
            tag_list: &mut tag_list,
        };
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
            let mut ctx = UndoContext {
                directory: &mut directory,
                tag_list: &mut tag_list,
            };
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
            let mut ctx = UndoContext {
                directory: &mut directory,
                tag_list: &mut tag_list,
            };
            history.undo(&mut ctx).unwrap();
        }
        assert_eq!(directory.selected_index(), Some(0));

        {
            let mut ctx = UndoContext {
                directory: &mut directory,
                tag_list: &mut tag_list,
            };
            history.redo(&mut ctx).unwrap();
        }
        assert_eq!(directory.selected_index(), Some(1));
    }

    // --- CheckTagsCommand ---

    #[test]
    fn checking_several_tags_is_one_step() {
        let store = make_store_with_tags(&["Action", "Beach", "Night"]);
        let snapshot = snapshot_with_tags(&["Action"]);
        let mut tag_list = TagList::new(store, snapshot);
        let mut dir = empty_directory();
        let beach = tag_list.tag_id_by_name("beach").unwrap();
        let night = tag_list.tag_id_by_name("Night").unwrap();
        tag_list.toggle_by_id(beach);
        tag_list.toggle_by_id(night);
        let mut history = History::new(50);
        history.push(Box::new(CheckTagsCommand {
            tag_ids: vec![beach, night],
        }));
        let checked = |list: &TagList<FakeAppStorage>| {
            ["Action", "Beach", "Night"].map(|name| {
                list.get_tag(list.tag_id_by_name(name).unwrap())
                    .unwrap()
                    .is_checked()
            })
        };
        {
            let mut ctx = UndoContext {
                directory: &mut dir,
                tag_list: &mut tag_list,
            };
            history.undo(&mut ctx).unwrap();
        }
        assert_eq!(
            checked(&tag_list),
            [true, false, false],
            "one undo, both off"
        );
        {
            let mut ctx = UndoContext {
                directory: &mut dir,
                tag_list: &mut tag_list,
            };
            history.redo(&mut ctx).unwrap();
        }
        assert_eq!(checked(&tag_list), [true, true, true]);
    }

    #[test]
    fn the_folders_own_tags_are_the_saved_ones() {
        let store = make_store_with_tags(&["Action", "Beach"]);
        let snapshot = snapshot_with_tags(&["Action", "only-in-the-name"]);
        let tag_list = TagList::new(store, snapshot);
        let mut names = tag_list.saved_tag_names();
        names.sort();
        assert_eq!(names, ["Action", "Beach"]);
        assert!(tag_list.tag_id_by_name("ONLY-IN-THE-NAME").is_some());
        assert!(tag_list.tag_id_by_name("missing").is_none());
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
        history.push(Box::new(ToggleTagCommand {
            tag_id: id,
            was_checked,
        }));

        {
            let mut ctx = UndoContext {
                directory: &mut dir,
                tag_list: &mut tag_list,
            };
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
        history.push(Box::new(ToggleTagCommand {
            tag_id: id,
            was_checked: true,
        }));

        {
            let mut ctx = UndoContext {
                directory: &mut dir,
                tag_list: &mut tag_list,
            };
            history.undo(&mut ctx).unwrap();
        }
        assert!(tag_list.get_tag(id).unwrap().is_checked());

        {
            let mut ctx = UndoContext {
                directory: &mut dir,
                tag_list: &mut tag_list,
            };
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
            let mut ctx = UndoContext {
                directory: &mut dir,
                tag_list: &mut tag_list,
            };
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
            let mut ctx = UndoContext {
                directory: &mut dir,
                tag_list: &mut tag_list,
            };
            history.undo(&mut ctx).unwrap();
        }
        assert_eq!(tag_list.file_snapshot().tags(), ["Action"]);

        {
            let mut ctx = UndoContext {
                directory: &mut dir,
                tag_list: &mut tag_list,
            };
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
            tag_id: id,
            tag_name: name,
            color_index,
            was_stored,
            was_starred,
            was_checked,
            sort_order,
        }));

        {
            let mut ctx = UndoContext {
                directory: &mut dir,
                tag_list: &mut tag_list,
            };
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
            tag_id: id,
            tag_name: name,
            color_index,
            was_stored,
            was_starred,
            was_checked,
            sort_order,
        }));

        {
            let mut ctx = UndoContext {
                directory: &mut dir,
                tag_list: &mut tag_list,
            };
            history.undo(&mut ctx).unwrap();
        }
        assert!(tag_list.get_tag(id).is_some());

        {
            let mut ctx = UndoContext {
                directory: &mut dir,
                tag_list: &mut tag_list,
            };
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
            let mut ctx = UndoContext {
                directory: &mut dir,
                tag_list: &mut tag_list,
            };
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
            let mut ctx = UndoContext {
                directory: &mut dir,
                tag_list: &mut tag_list,
            };
            history.undo(&mut ctx).unwrap();
        }
        assert!(tag_list.get_tag(id).is_none());

        {
            let mut ctx = UndoContext {
                directory: &mut dir,
                tag_list: &mut tag_list,
            };
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
        history.push(Box::new(SaveTagCommand {
            tag_id: id,
            color_index,
        }));

        {
            let mut ctx = UndoContext {
                directory: &mut dir,
                tag_list: &mut tag_list,
            };
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
        history.push(Box::new(SaveTagCommand {
            tag_id: id,
            color_index,
        }));

        {
            let mut ctx = UndoContext {
                directory: &mut dir,
                tag_list: &mut tag_list,
            };
            history.undo(&mut ctx).unwrap();
        }
        assert!(!tag_list.get_tag(id).unwrap().is_stored());

        {
            let mut ctx = UndoContext {
                directory: &mut dir,
                tag_list: &mut tag_list,
            };
            history.redo(&mut ctx).unwrap();
        }
        assert!(tag_list.get_tag(id).unwrap().is_stored());
    }

    /// Issue #97: saving a tag that is stored already changes nothing, so no step is recorded
    /// and undo leaves the tag stored.
    #[test]
    fn saving_a_stored_tag_again_is_not_a_step() {
        let store = FakeAppStorage::new();
        let mut tag_list = TagList::new(store, FileSnapshot::default());
        let mut dir = empty_directory();
        let id = tag_list.create_new_tag("NewTag").unwrap();
        let mut history = History::new(50);

        // A real save is a step.
        assert!(tag_list.save_tag_once(id).unwrap());
        let color_index = tag_list.get_tag(id).unwrap().color_index();
        history.push(Box::new(SaveTagCommand {
            tag_id: id,
            color_index,
        }));
        let mut ctx = UndoContext {
            directory: &mut dir,
            tag_list: &mut tag_list,
        };
        history.undo(&mut ctx).unwrap();
        assert!(!ctx.tag_list.get_tag(id).unwrap().is_stored());
        history.redo(&mut ctx).unwrap();
        assert!(ctx.tag_list.get_tag(id).unwrap().is_stored());

        // Saving it again reports "already saved": the caller pushes nothing.
        history = History::new(50);
        assert!(!ctx.tag_list.save_tag_once(id).unwrap());
        assert!(!history.can_undo());
        assert!(ctx.tag_list.get_tag(id).unwrap().is_stored());
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
        history.push(Box::new(StarTagCommand {
            tag_id: id,
            was_starred: false,
        }));

        {
            let mut ctx = UndoContext {
                directory: &mut dir,
                tag_list: &mut tag_list,
            };
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
        history.push(Box::new(StarTagCommand {
            tag_id: id,
            was_starred: false,
        }));

        {
            let mut ctx = UndoContext {
                directory: &mut dir,
                tag_list: &mut tag_list,
            };
            history.undo(&mut ctx).unwrap();
        }
        assert!(!tag_list.get_tag(id).unwrap().is_starred());

        {
            let mut ctx = UndoContext {
                directory: &mut dir,
                tag_list: &mut tag_list,
            };
            history.redo(&mut ctx).unwrap();
        }
        assert!(tag_list.get_tag(id).unwrap().is_starred());
    }

    #[test]
    fn navigate_command_undo_fails_gracefully_when_file_not_found() {
        let dir_path = PathBuf::from("C:/test");
        let files = vec![File::from_path(
            dir_path.join("a.mp4"),
            SystemTime::UNIX_EPOCH,
        )];
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
            let mut ctx = UndoContext {
                directory: &mut directory,
                tag_list: &mut tag_list,
            };
            history.undo(&mut ctx).unwrap(); // undo selects file_a_id — succeeds
        }
        // redo should fail: to_file_id is bogus
        let mut ctx = UndoContext {
            directory: &mut directory,
            tag_list: &mut tag_list,
        };
        let result = history.redo(&mut ctx);
        assert!(result.is_err());
        // Stack unchanged after error
        assert!(history.can_redo());
    }

    // --- Markers ---

    fn marker_list() -> TagList<FakeAppStorage> {
        let mut snapshot = snapshot_with_tags(&[]);
        snapshot.set_markers(Some(Vec::new()));
        TagList::new(FakeAppStorage::new(), snapshot)
    }

    fn run(
        history: &mut History<FakeAppStorage, FakeAppStorage>,
        tag_list: &mut TagList<FakeAppStorage>,
        undo: bool,
    ) {
        let mut dir = empty_directory();
        let mut ctx = UndoContext {
            directory: &mut dir,
            tag_list,
        };
        if undo {
            history.undo(&mut ctx).unwrap();
        } else {
            history.redo(&mut ctx).unwrap();
        }
    }

    #[test]
    fn undoing_an_added_marker_removes_it_and_redo_brings_back_its_name() {
        let mut tag_list = marker_list();
        let mut history = History::new(50);
        let marker = Marker::new(1_000);
        let guid = marker.guid.clone().unwrap();
        tag_list.add_marker(marker.clone());
        history.push(Box::new(AddMarkerCommand { marker }));
        tag_list.update_marker(&guid, |m| m.name = "Typed after adding".to_string());

        run(&mut history, &mut tag_list, true);
        assert_eq!(tag_list.markers().unwrap().len(), 0);
        run(&mut history, &mut tag_list, false);
        assert_eq!(tag_list.marker(&guid).unwrap().name, "Typed after adding");
        // Redo again after another undo cannot duplicate it.
        run(&mut history, &mut tag_list, true);
        run(&mut history, &mut tag_list, false);
        assert_eq!(tag_list.markers().unwrap().len(), 1);
    }

    #[test]
    fn undoing_a_delete_restores_the_marker() {
        let mut tag_list = marker_list();
        let mut history = History::new(50);
        let mut marker = Marker::new(2_000);
        marker.name = "Keep".to_string();
        let guid = marker.guid.clone().unwrap();
        tag_list.add_marker(marker);
        let removed = tag_list.remove_marker(&guid).unwrap();
        history.push(Box::new(DeleteMarkerCommand { marker: removed }));

        run(&mut history, &mut tag_list, true);
        assert_eq!(tag_list.marker(&guid).unwrap().name, "Keep");
        run(&mut history, &mut tag_list, false);
        assert!(tag_list.marker(&guid).is_none());
    }

    #[test]
    fn color_and_duration_changes_undo_and_redo() {
        let mut tag_list = marker_list();
        let mut history = History::new(50);
        let marker = Marker::new(0);
        let guid = marker.guid.clone().unwrap();
        tag_list.add_marker(marker);
        tag_list.update_marker(&guid, |m| m.color = MarkerColor::Red);
        history.push(Box::new(SetMarkerColorCommand {
            guid: guid.clone(),
            old: MarkerColor::Green,
            new: MarkerColor::Red,
        }));
        tag_list.update_marker(&guid, |m| m.duration_ms = 1_500);
        history.push(Box::new(SetMarkerSpanCommand {
            guid: guid.clone(),
            old: (0, 0),
            new: (0, 1_500),
        }));

        run(&mut history, &mut tag_list, true);
        assert_eq!(tag_list.marker(&guid).unwrap().duration_ms, 0);
        run(&mut history, &mut tag_list, true);
        assert_eq!(tag_list.marker(&guid).unwrap().color, MarkerColor::Green);
        run(&mut history, &mut tag_list, false);
        run(&mut history, &mut tag_list, false);
        let back = tag_list.marker(&guid).unwrap();
        assert_eq!((back.color, back.duration_ms), (MarkerColor::Red, 1_500));
    }

    #[test]
    fn a_range_added_moved_and_resized_undoes_step_by_step() {
        let mut tag_list = marker_list();
        let mut history = History::new(50);
        // Held F2: one step adds the marker with its length.
        let mut marker = Marker::new(41_000);
        marker.duration_ms = 6_000;
        let guid = marker.guid.clone().unwrap();
        tag_list.add_marker(marker.clone());
        history.push(Box::new(AddMarkerCommand { marker }));
        // A handle drag moves the start; another resizes it back to a point.
        tag_list.update_marker(&guid, |m| (m.start_ms, m.duration_ms) = (40_000, 7_000));
        history.push(Box::new(SetMarkerSpanCommand {
            guid: guid.clone(),
            old: (41_000, 6_000),
            new: (40_000, 7_000),
        }));
        tag_list.update_marker(&guid, |m| m.duration_ms = 0);
        history.push(Box::new(SetMarkerSpanCommand {
            guid: guid.clone(),
            old: (40_000, 7_000),
            new: (40_000, 0),
        }));
        let span = |tag_list: &TagList<FakeAppStorage>| {
            tag_list.marker(&guid).map(|m| (m.start_ms, m.duration_ms))
        };

        run(&mut history, &mut tag_list, true);
        assert_eq!(span(&tag_list), Some((40_000, 7_000)));
        run(&mut history, &mut tag_list, true);
        assert_eq!(span(&tag_list), Some((41_000, 6_000)));
        run(&mut history, &mut tag_list, true);
        assert_eq!(span(&tag_list), None);
        run(&mut history, &mut tag_list, false);
        assert_eq!(span(&tag_list), Some((41_000, 6_000)));
        run(&mut history, &mut tag_list, false);
        run(&mut history, &mut tag_list, false);
        assert_eq!(span(&tag_list), Some((40_000, 0)));
    }

    #[test]
    fn undoing_a_paste_keeps_the_markers_added_after_it() {
        let mut tag_list = marker_list();
        let mut history = History::new(50);
        let before = tag_list.file_snapshot();
        let after = snapshot_with_tags(&["Pasted"]);
        tag_list.reinitialize_from_snapshot(after.clone());
        history.push(Box::new(PasteTagsCommand {
            snapshot_before: before,
            snapshot_after: after,
        }));
        let marker = Marker::new(500);
        tag_list.add_marker(marker.clone());

        run(&mut history, &mut tag_list, true);
        run(&mut history, &mut tag_list, false);
        assert_eq!(tag_list.markers().unwrap(), [marker]);
    }

    #[test]
    fn the_snapshot_of_a_tag_list_keeps_its_markers() {
        let mut snapshot = snapshot_with_tags(&["A"]);
        let markers = vec![Marker::new(3_000), Marker::new(1_000)];
        snapshot.set_markers(Some(markers.clone()));
        let tag_list = TagList::new(FakeAppStorage::new(), snapshot);
        let back = tag_list.file_snapshot();
        let mut sorted = markers;
        crate::sort_markers(&mut sorted);
        assert_eq!(back.markers(), Some(sorted.as_slice()));
        assert_eq!(
            TagList::new(FakeAppStorage::new(), snapshot_with_tags(&[]))
                .file_snapshot()
                .markers(),
            None
        );
    }

    /// Turning a video: undo turns it back, redo turns it again. Tests run with the in-memory
    /// file tagger, so the turns are kept in memory; the file tagger's own tests cover disk.
    #[test]
    fn undo_and_redo_turn_the_video() {
        use crate::undo::RotateVideoCommand;
        use crate::{FileTagger, Rotation};

        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/wide.mov");
        let dir = crate::test_support::fresh_dir("undo-rotate");
        let path = dir.join("clip.mov");
        std::fs::copy(fixture, &path).expect("copy");

        let file = File::from_path(path.clone(), SystemTime::UNIX_EPOCH);
        let id = file.id();
        let mut directory = Directory::with_files(&dir, vec![file], FakeAppStorage::new());
        let mut tag_list = TagList::new(FakeAppStorage::new(), FileSnapshot::default());
        let mut history = History::new(50);

        FileTagger::rotate_video(&path, -1).expect("rotate");
        history.push(Box::new(RotateVideoCommand {
            file: id,
            path: path.clone(),
            quarter_turns: -1,
        }));
        let degrees = || FileTagger::video_rotation(&path).map(Rotation::degrees);
        assert_eq!(degrees(), Ok(270));

        assert!(history.undo_turns_a_video());
        assert!(!history.redo_turns_a_video());
        let mut ctx = UndoContext {
            directory: &mut directory,
            tag_list: &mut tag_list,
        };
        history.undo(&mut ctx).expect("undo");
        assert_eq!(degrees(), Ok(0));
        assert!(history.redo_turns_a_video());
        history.redo(&mut ctx).expect("redo");
        assert_eq!(degrees(), Ok(270));
    }

    #[test]
    fn a_step_says_whether_it_edits_the_open_video_or_moves_to_another_file() {
        let mut history: History<FakeAppStorage, FakeAppStorage> = History::new(50);
        assert!(!history.undo_edits_open_video());
        history.push(Box::new(AddMarkerCommand {
            marker: Marker::new(1_000),
        }));
        assert!(history.undo_edits_open_video());
        assert!(!history.undo_switches_file());
        history.push(Box::new(NavigateFileCommand {
            file_id: File::from_path(PathBuf::from("/x/a.mp4"), SystemTime::UNIX_EPOCH).id(),
            to_file_id: File::from_path(PathBuf::from("/x/b.mp4"), SystemTime::UNIX_EPOCH).id(),
            path_before: PathBuf::from("/x/a.mp4"),
            path_after: PathBuf::from("/x/a.mp4"),
            snapshot_before: FileSnapshot::default(),
            snapshot_after: FileSnapshot::default(),
        }));
        assert!(!history.undo_edits_open_video());
        assert!(history.undo_switches_file());
        assert!(!history.redo_edits_open_video());
        assert!(!history.redo_switches_file());
    }

    // --- Issue #139: rename, comment, marker name, sync ---

    #[test]
    fn rename_command_puts_the_old_and_new_name_back() {
        let mut tag_list = TagList::new(FakeAppStorage::new(), snapshot_with_tags(&["a"]));
        let before = tag_list.file_snapshot();
        let name_before = before.file_name();
        let after = FileSnapshot::new(vec!["a".into()], "renamed", "mp4", "renamed.mp4");
        let name_after = after.file_name();
        assert_ne!(name_before, name_after);
        tag_list.reinitialize_from_snapshot(after.clone());
        let mut history = History::new(50);
        history.push(Box::new(RenameFileCommand {
            snapshot_before: before,
            snapshot_after: after,
        }));

        run(&mut history, &mut tag_list, true);
        assert_eq!(tag_list.file_snapshot().file_name(), name_before);
        run(&mut history, &mut tag_list, false);
        assert_eq!(tag_list.file_snapshot().file_name(), name_after);
    }

    #[test]
    fn comment_command_restores_the_comment_and_redoes_it() {
        let mut tag_list = TagList::new(FakeAppStorage::new(), snapshot_with_tags(&[]));
        let before = tag_list.file_snapshot();
        tag_list.set_comment("a note".to_string());
        let after = tag_list.file_snapshot();
        let mut history = History::new(50);
        history.push(Box::new(SetCommentCommand {
            snapshot_before: before,
            snapshot_after: after,
        }));

        run(&mut history, &mut tag_list, true);
        assert_eq!(tag_list.comment(), "");
        run(&mut history, &mut tag_list, false);
        assert_eq!(tag_list.comment(), "a note");
    }

    #[test]
    fn marker_name_command_undoes_and_redoes_the_name() {
        let mut tag_list = marker_list();
        let marker = Marker::new(0);
        let guid = marker.guid.clone().unwrap();
        tag_list.add_marker(marker);
        tag_list.update_marker(&guid, |m| m.name = "hello".to_string());
        let mut history = History::new(50);
        history.push(Box::new(SetMarkerNameCommand {
            guid: guid.clone(),
            old: String::new(),
            new: "hello".to_string(),
        }));

        run(&mut history, &mut tag_list, true);
        assert_eq!(tag_list.marker(&guid).unwrap().name, "");
        run(&mut history, &mut tag_list, false);
        assert_eq!(tag_list.marker(&guid).unwrap().name, "hello");
    }

    #[test]
    fn marker_text_command_undoes_and_redoes_the_name_and_comment_together() {
        let mut tag_list = marker_list();
        let mut marker = Marker::new(0);
        marker.comment = "mine".to_string();
        let guid = marker.guid.clone().unwrap();
        tag_list.add_marker(marker);
        let new = ("Lion".to_string(), "mine\nA lion walks past.".to_string());
        tag_list.update_marker(&guid, |m| (m.name, m.comment) = new.clone());
        let mut history = History::new(50);
        history.push(Box::new(SetMarkerTextCommand {
            guid: guid.clone(),
            old: (String::new(), "mine".to_string()),
            new: new.clone(),
        }));

        run(&mut history, &mut tag_list, true);
        let undone = tag_list.marker(&guid).unwrap();
        assert_eq!(
            (undone.name.as_str(), undone.comment.as_str()),
            ("", "mine")
        );
        run(&mut history, &mut tag_list, false);
        let redone = tag_list.marker(&guid).unwrap();
        assert_eq!((redone.name.clone(), redone.comment.clone()), new);
    }

    #[test]
    fn sync_command_restores_the_orders_and_the_lock() {
        let store = make_store_with_tags(&["Action", "Comedy", "Summer"]);
        let mut tag_list = TagList::new(store, snapshot_with_tags(&["Action", "Comedy", "Summer"]));
        tag_list.set_sync_locked(false);
        let id_action = tag_list.checked_tag_id_at(0).unwrap();
        // Make the file name's order differ from the grid's, then sync it into the grid.
        tag_list.reorder_tag_to_index(id_action, 2);
        let before = tag_list.order_state();
        let grid_before = tag_list.filtered_display_tag_ids().to_vec();
        tag_list.sync_selected_to_display().unwrap();
        tag_list.set_sync_locked(true);
        let after = tag_list.order_state();
        assert_ne!(tag_list.filtered_display_tag_ids(), grid_before);

        let mut history = History::new(50);
        history.push(Box::new(SyncTagOrderCommand { before, after }));
        run(&mut history, &mut tag_list, true);
        assert_eq!(tag_list.filtered_display_tag_ids(), grid_before);
        assert!(!tag_list.sync_locked());
        run(&mut history, &mut tag_list, false);
        assert_ne!(tag_list.filtered_display_tag_ids(), grid_before);
        assert!(tag_list.sync_locked());
    }

    #[test]
    fn probe_stale_ids() {
        let store = make_store_with_tags(&["Action", "Comedy"]);
        let mut tag_list = TagList::new(store, snapshot_with_tags(&["Action", "Zed", "Comedy"]));
        let ids_before: Vec<_> = tag_list.filtered_display_tag_ids().to_vec();
        let n = (0..5).filter_map(|i| tag_list.checked_tag_id_at(i)).count();
        let st = tag_list.order_state();
        let snap = tag_list.file_snapshot();
        tag_list.reinitialize_from_snapshot(snap);
        let ids_mid: Vec<_> = tag_list.filtered_display_tag_ids().to_vec();
        tag_list.restore_order_state(st).unwrap();
        let ids_after: Vec<_> = tag_list.filtered_display_tag_ids().to_vec();
        let n2 = (0..5).filter_map(|i| tag_list.checked_tag_id_at(i)).count();
        eprintln!(
            "PROBE before {} mid {} after {} checked {} -> {}",
            ids_before.len(),
            ids_mid.len(),
            ids_after.len(),
            n,
            n2
        );
        eprintln!("PROBE same ids mid vs after {}", ids_mid == ids_after);
    }

    #[test]
    fn order_state_survives_a_rebuild_with_an_unsaved_tag() {
        let store = make_store_with_tags(&["Action", "Comedy"]);
        let snapshot = snapshot_with_tags(&["Action", "Comedy", "Zed"]);
        let mut tag_list = TagList::new(store, snapshot.clone());
        let state = tag_list.order_state();
        // A rebuild gives the unsaved tag a new id.
        tag_list.reinitialize_from_snapshot(snapshot);
        tag_list.restore_order_state(state).unwrap();
        assert_eq!(tag_list.file_snapshot().tags(), ["Action", "Comedy", "Zed"]);
        assert_eq!(tag_list.filtered_display_tag_ids().len(), 3);
    }

    #[test]
    fn setting_both_in_out_points_is_one_undo_step() {
        let mut tag_list = marker_list();
        let mut history = History::new(50);
        tag_list.set_segment_start_secs(Some(1.0));
        let old = tag_list.file_snapshot().segment();
        let new = crate::Segment {
            start: Some(3.0),
            end: Some(12.0),
        };
        tag_list.set_segment_start_secs(new.start);
        tag_list.set_segment_end_secs(new.end);
        history.push(Box::new(SetSegmentCommand { old, new }));

        run(&mut history, &mut tag_list, true);
        assert_eq!(
            tag_list.file_snapshot().segment(),
            crate::Segment {
                start: Some(1.0),
                end: None,
            },
            "one undo brings back both points as they were"
        );
        run(&mut history, &mut tag_list, false);
        assert_eq!(tag_list.file_snapshot().segment(), new);
    }

    // --- NavigateFileCommand renames on a real folder ---

    /// A folder with `names` as files (content = the name), and a command that renamed
    /// `before` to `after`. Returns the folder, the directory and the command.
    fn rename_fixture(
        label: &str,
        before: &str,
        after: &str,
    ) -> (
        PathBuf,
        Directory<FakeAppStorage>,
        NavigateFileCommand,
        FileId,
    ) {
        let dir_path = crate::test_support::fresh_dir(&format!("undo-{label}"));
        let file = File::from_path(dir_path.join(after), SystemTime::UNIX_EPOCH);
        let id = file.id();
        let directory = Directory::with_files(&dir_path, vec![file], FakeAppStorage::new());
        let snap = FileSnapshot::new(vec![], "x", "mp4", before);
        let cmd = NavigateFileCommand {
            file_id: id,
            to_file_id: id,
            path_before: dir_path.join(before),
            path_after: dir_path.join(after),
            snapshot_before: snap.clone(),
            snapshot_after: snap,
        };
        (dir_path, directory, cmd, id)
    }

    fn write(path: &std::path::Path, text: &str) {
        std::fs::write(path, text).expect("write");
    }

    #[test]
    fn undo_of_a_rename_refuses_to_overwrite_another_clip() {
        let (dir_path, mut directory, cmd, _) =
            rename_fixture("undo-clobber", "foo.mp4", "foo.GOAT.mp4");
        write(&dir_path.join("foo.GOAT.mp4"), "clip A");
        // Clip B was renamed onto the name clip A left free.
        write(&dir_path.join("foo.mp4"), "clip B");
        let mut tag_list = TagList::new(FakeAppStorage::new(), FileSnapshot::default());
        let mut history = History::new(50);
        history.push(Box::new(cmd));

        let mut ctx = UndoContext {
            directory: &mut directory,
            tag_list: &mut tag_list,
        };
        let err = history.undo(&mut ctx).expect_err("must refuse");
        assert!(matches!(err, crate::undo::UndoError::NameTaken(_)), "{err}");
        assert_eq!(
            std::fs::read_to_string(dir_path.join("foo.mp4")).unwrap(),
            "clip B"
        );
        assert_eq!(
            std::fs::read_to_string(dir_path.join("foo.GOAT.mp4")).unwrap(),
            "clip A"
        );
        // The refused step is not lost: it can be tried again, and nothing moved to redo.
        assert!(history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn a_refused_undo_works_once_the_name_is_free() {
        let (dir_path, mut directory, cmd, _) =
            rename_fixture("undo-retry", "foo.mp4", "foo.GOAT.mp4");
        write(&dir_path.join("foo.GOAT.mp4"), "clip A");
        write(&dir_path.join("foo.mp4"), "clip B");
        let mut tag_list = TagList::new(FakeAppStorage::new(), FileSnapshot::default());
        let mut history = History::new(50);
        history.push(Box::new(cmd));
        let mut ctx = UndoContext {
            directory: &mut directory,
            tag_list: &mut tag_list,
        };
        assert!(history.undo(&mut ctx).is_err());
        std::fs::remove_file(dir_path.join("foo.mp4")).unwrap();
        history.undo(&mut ctx).expect("name is free now");
        assert_eq!(
            std::fs::read_to_string(dir_path.join("foo.mp4")).unwrap(),
            "clip A"
        );
        assert!(history.can_redo());
    }

    #[test]
    fn redo_of_a_rename_refuses_to_overwrite_another_clip() {
        let (dir_path, mut directory, cmd, _) =
            rename_fixture("redo-clobber", "foo.mp4", "foo.GOAT.mp4");
        write(&dir_path.join("foo.mp4"), "clip A");
        // Something else took the name the redo would give clip A.
        write(&dir_path.join("foo.GOAT.mp4"), "clip C");
        let mut tag_list = TagList::new(FakeAppStorage::new(), FileSnapshot::default());
        let mut history = History::new(50);
        history.push(Box::new(cmd));
        let mut ctx = UndoContext {
            directory: &mut directory,
            tag_list: &mut tag_list,
        };
        // Put the command on the redo stack without touching the files: undo needs the
        // renamed file's name free, so move clip C away for that step and back.
        std::fs::rename(dir_path.join("foo.GOAT.mp4"), dir_path.join("c.tmp")).unwrap();
        std::fs::rename(dir_path.join("foo.mp4"), dir_path.join("foo.GOAT.mp4")).unwrap();
        history.undo(&mut ctx).expect("undo");
        write(&dir_path.join("foo.GOAT.mp4"), "clip C");

        let err = history.redo(&mut ctx).expect_err("must refuse");
        assert!(matches!(err, crate::undo::UndoError::NameTaken(_)), "{err}");
        assert_eq!(
            std::fs::read_to_string(dir_path.join("foo.GOAT.mp4")).unwrap(),
            "clip C"
        );
        assert_eq!(
            std::fs::read_to_string(dir_path.join("foo.mp4")).unwrap(),
            "clip A"
        );
        assert!(history.can_redo());
    }

    #[test]
    fn undo_and_redo_of_a_rename_move_the_comment_along() {
        let (dir_path, mut directory, cmd, _) =
            rename_fixture("comment-moves", "foo.mp4", "foo.GOAT.mp4");
        let (before, after) = (dir_path.join("foo.mp4"), dir_path.join("foo.GOAT.mp4"));
        write(&after, "clip A");
        crate::comment::save_comment(&after, "nice take");
        let mut tag_list = TagList::new(FakeAppStorage::new(), FileSnapshot::default());
        let mut history = History::new(50);
        history.push(Box::new(cmd));
        let mut ctx = UndoContext {
            directory: &mut directory,
            tag_list: &mut tag_list,
        };

        history.undo(&mut ctx).expect("undo");
        assert_eq!(crate::comment::load_comment(&before), "nice take");
        assert!(!crate::comment::comment_path(&after).exists());

        history.redo(&mut ctx).expect("redo");
        assert_eq!(crate::comment::load_comment(&after), "nice take");
        assert!(!crate::comment::comment_path(&before).exists());
    }

    #[test]
    fn undo_refuses_when_the_comment_name_is_taken() {
        let (dir_path, mut directory, cmd, _) =
            rename_fixture("comment-taken", "foo.mp4", "foo.GOAT.mp4");
        let (before, after) = (dir_path.join("foo.mp4"), dir_path.join("foo.GOAT.mp4"));
        write(&after, "clip A");
        crate::comment::save_comment(&after, "comment of A");
        // No clip is called foo.mp4, but a comment is left under its name.
        crate::comment::save_comment(&before, "stray comment");
        let mut tag_list = TagList::new(FakeAppStorage::new(), FileSnapshot::default());
        let mut history = History::new(50);
        history.push(Box::new(cmd));
        let mut ctx = UndoContext {
            directory: &mut directory,
            tag_list: &mut tag_list,
        };

        let err = history.undo(&mut ctx).expect_err("must refuse");
        assert!(matches!(err, crate::undo::UndoError::NameTaken(_)), "{err}");
        assert!(after.exists(), "the clip stays where it was");
        assert_eq!(crate::comment::load_comment(&before), "stray comment");
        assert_eq!(crate::comment::load_comment(&after), "comment of A");
    }

    #[test]
    fn undo_and_redo_of_a_rename_move_every_sidecar_the_save_moves() {
        let (dir_path, mut directory, cmd, _) =
            rename_fixture("all-sidecars", "foo.mp4", "foo.GOAT.mp4");
        let (before, after) = (dir_path.join("foo.mp4"), dir_path.join("foo.GOAT.mp4"));
        write(&after, "clip A");
        crate::comment::save_comment(&after, "nice take");
        let sidecars = |video: &std::path::Path| {
            let mut all = crate::subtitles::subtitle_candidates(video);
            all.push(crate::subtitles::transcript_path(video));
            all
        };
        for p in sidecars(&after) {
            write(&p, "sidecar");
        }
        let snap = |clip: &str| dir_path.join(format!("{clip}.snap.00-00-10-936.jpg"));
        write(&snap("foo.GOAT.mp4"), "jpg");
        // Another clip whose name starts with this one keeps its screenshot.
        let other = snap("foo.GOAT.mp4.bak");
        write(&other, "other");
        let mut tag_list = TagList::new(FakeAppStorage::new(), FileSnapshot::default());
        let mut history = History::new(50);
        history.push(Box::new(cmd));
        let mut ctx = UndoContext {
            directory: &mut directory,
            tag_list: &mut tag_list,
        };

        history.undo(&mut ctx).expect("undo");
        assert!(before.exists() && !after.exists());
        assert!(crate::comment::comment_path(&before).exists());
        for (old, new) in sidecars(&after).iter().zip(sidecars(&before).iter()) {
            assert!(new.exists() && !old.exists(), "{old:?} -> {new:?}");
        }
        assert!(snap("foo.mp4").exists() && !snap("foo.GOAT.mp4").exists());
        assert!(other.exists());

        history.redo(&mut ctx).expect("redo");
        assert!(after.exists() && !before.exists());
        assert!(crate::comment::comment_path(&after).exists());
        for (old, new) in sidecars(&before).iter().zip(sidecars(&after).iter()) {
            assert!(new.exists() && !old.exists(), "{old:?} -> {new:?}");
        }
        assert!(snap("foo.GOAT.mp4").exists() && !snap("foo.mp4").exists());
        assert!(other.exists());
    }

    #[test]
    fn undo_refuses_when_a_screenshot_name_is_taken() {
        let (dir_path, mut directory, cmd, _) =
            rename_fixture("snap-taken", "foo.mp4", "foo.GOAT.mp4");
        let after = dir_path.join("foo.GOAT.mp4");
        write(&after, "clip A");
        let snap = |clip: &str| dir_path.join(format!("{clip}.snap.00-00-01-000.jpg"));
        write(&snap("foo.GOAT.mp4"), "A");
        write(&snap("foo.mp4"), "stray");
        let mut tag_list = TagList::new(FakeAppStorage::new(), FileSnapshot::default());
        let mut history = History::new(50);
        history.push(Box::new(cmd));
        let mut ctx = UndoContext {
            directory: &mut directory,
            tag_list: &mut tag_list,
        };
        let err = history.undo(&mut ctx).expect_err("must refuse");
        assert!(matches!(err, crate::undo::UndoError::NameTaken(_)), "{err}");
        assert!(after.exists());
        assert_eq!(std::fs::read(snap("foo.mp4")).unwrap(), b"stray");
        assert_eq!(std::fs::read(snap("foo.GOAT.mp4")).unwrap(), b"A");
    }

    #[test]
    fn a_rename_of_a_clip_without_screenshots_moves_nothing_else() {
        let (dir_path, mut directory, cmd, _) =
            rename_fixture("no-snaps", "foo.mp4", "foo.GOAT.mp4");
        write(&dir_path.join("foo.GOAT.mp4"), "clip A");
        let mut tag_list = TagList::new(FakeAppStorage::new(), FileSnapshot::default());
        let mut history = History::new(50);
        history.push(Box::new(cmd));
        let mut ctx = UndoContext {
            directory: &mut directory,
            tag_list: &mut tag_list,
        };
        history.undo(&mut ctx).expect("undo");
        assert!(dir_path.join("foo.mp4").exists());
    }
}
