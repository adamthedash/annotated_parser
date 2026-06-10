# Annotated Parser
A parser combinator library for experimental parser creation. It creates annotations over the data on execution which can be used to visualise the parsers.  
Example of where it's used in my [hex viewer](https://github.com/adamthedash/hex_viewer) project.  

### Goals
- Inspect parser structure without running it on any data
- Annotate parser execution on data whether successful or not
- Minimal additional boilerplate over defining the parser itself
- Ergonomic use of parsers once they are defined
- Strong type inference at call site
- Good performance in non-annotating mode

# AI Disclaimer
LLMs use has been limited to:
- Documentation
