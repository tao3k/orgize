# Expected

Project-wide Org metadata query uses the lexical prefilter to reject literal
no-hit documents before parser-owned element indexing. Parser-only terms such as
`heading` still pass through to parser indexing.
