use syn::buffer::Cursor;

pub trait CursorParse: Sized
{
	fn parse_from_cursor (cursor: Cursor <'_>) -> Option <(Self, Cursor <'_>)>;
}
