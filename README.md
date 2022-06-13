# Kibi w/Melda (Simple collaborative editor with Melda CRDT)

[Kibi](https://github.com/ilai-deutel/kibi) is  configurable text editor with UTF-8 support, incremental search, syntax
highlighting, line numbers and more, written in Rust with minimal dependencies. In this fork, simple collaboration features based on 
[Melda](https://github.com/slashdotted/libmelda) have been implemented. The implemented paradigm is [save and refresh](https://support.microsoft.com/en-us/office/save-and-refresh-documents-e0baba43-d843-459b-95dd-d1973b65a2aa): when the user saves the local copy, changes from other users are merged. It is also possible to merge changes without saving the document (using the 'Refresh' keyboard shortcut).

# What is Melda?

Melda is a Delta-State JSON CRDT. CRDTs, which stand for Conflict-free Replicated Data Types, are data structures which can be replicated (copied) across multiple computers in a network. Each replica can be individually and concurrently updated without the need for central coordination or synchronization. Updates made on each replica can be merged at any time.

There exist different types of CRDTs: operation-based CRDTs (which generate and exchange update operations between replicas), state-based CRDTS (which exchange and merge the full state of each replica) and delta-state CRDT, such as Melda, (which exchange only the differences between versions, or states, of the data type).

Melda natively supports the JSON data format and provides a way to synchronize changes made to arbitrary JSON documents.

## Usage of Kibi w/Melda
For the majority of the functionalities, this version works as the main version of [Kibi](https://github.com/ilai-deutel/kibi). 
However, it is now possible to use one of the following CRDT storage bakcends when opening a document from the command line:
| Storage type      | Example path                                              | Description |
| ----------------- | ------------------------------------------------------------- | -------------------------- |
| Folder (file://)           | file://$(pwd)/mycrdtdocument                   | The absolute path of a folder (can be on a network share) |
| Folder w/compression (file+flate://)           | file+flate://$(pwd)/mycrdtdocument     | The absolute path of a folder (can be on a network share) |
| [Solid](https://solidproject.org/) Pod (solid://)           | solid://anuser.solidcommunity.net/mycrdtdocument | The URL of a [Solid](https://solidproject.org/) Pod |
| [Solid](https://solidproject.org/) Pod w/compression (solid+flate://)            | solid+flate://anuser.solidcommunity.net/mycrdtdocument  | The URL of a [Solid](https://solidproject.org/) Pod |                                                      |
 
 For [Solid](https://solidproject.org/) Pod's access, a username and password will be asked. Compression uses a DEFLATE-based algorithm.


### Keyboard shortcuts

| Keyboard shortcut | Description                                                   |
| ----------------- | ------------------------------------------------------------- |
| Ctrl-F            | Incremental search; use arrows to navigate                    |
| Ctrl-S            | Save the buffer to the current file, or specify the file path |
| Ctrl-G            | Go to `<line number>[:<column number>]` position              |
| Ctrl-Q            | Quit                                                          |
| Ctrl-D            | Duplicate the current row                                     |
| Ctrl-E            | Execute an external command and paste its output              |
| Ctrl-R            | Remove an entire line                                         |
| **Ctrl-P**         | Refresh document (integrate changes from other users)        |
| **Ctrl-N**         | Save document with a new name / path        |

