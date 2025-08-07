use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub enum Action {
    Shortcut(String),
    Text(String),
    Line(String),
    Pause(u64),
    OpenUrl(String),
    CustomHomeAction,
    Command(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionThread {
    Main,
    Background,
}

impl Action {
    pub fn thread(&self) -> ExecutionThread {
        match self {
            Action::CustomHomeAction => ExecutionThread::Main,
            Action::OpenUrl(_) => ExecutionThread::Background,
            Action::Command(_) => ExecutionThread::Background,
            _ => ExecutionThread::Background,
        }
    }

    pub fn is_delayed(&self) -> bool {
        matches!(self, Action::Pause(_))
    }
}

/// Internal utility trait for action collections
pub trait ActionList {
    fn is_order_valid(&self) -> bool;
    fn is_delayed(&self) -> bool;
    fn split(&self) -> (Vec<Action>, Vec<Action>);
}

impl ActionList for Vec<Action> {
    fn is_order_valid(&self) -> bool {
        fn find_edge_indexes(vect: &Vec<Action>) -> (Option<usize>, Option<usize>) {
            let mut last_background_index = None;
            let mut first_main_index = None;

            for (index, action) in vect.iter().enumerate() {
                if action.thread() == ExecutionThread::Background {
                    last_background_index = Some(index);
                } else if action.thread() == ExecutionThread::Main {
                    first_main_index = Some(index);
                    break;
                }
            }
            (last_background_index, first_main_index)
        }

        let (last_background_index, first_main_index) = find_edge_indexes(self);

        match (last_background_index, first_main_index) {
            (Some(bg), Some(main)) => bg < main,
            _ => true,
        }
    }

    fn is_delayed(&self) -> bool {
        self.iter().any(|action| action.is_delayed())
    }

    fn split(&self) -> (Vec<Action>, Vec<Action>) {
        let mut background_actions = Vec::new();
        let mut main_actions = Vec::new();

        for action in self.iter() {
            match action.thread() {
                ExecutionThread::Background => background_actions.push(action.clone()),
                ExecutionThread::Main => main_actions.push(action.clone()),
            }
        }

        (background_actions, main_actions)
    }
}