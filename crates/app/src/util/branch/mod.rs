mod name;

pub use name::{
    branch_type_from_branch_name, generate_branch_name_from_jira,
    generate_branch_name_from_template, select_branch_type, to_slug,
};
