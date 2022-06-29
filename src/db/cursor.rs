use super::table::Table;

pub struct Cursor {
    table: Table,
    page_num: usize,
    cell_num: usize,
    end_of_table: bool,
}
