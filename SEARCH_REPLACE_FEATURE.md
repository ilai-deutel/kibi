# Search and Replace Feature

## Overview
A new search and replace feature has been added to the kibi text editor, building on top of the existing search functionality (Ctrl+F).

## How to Use

### Basic Search (Existing Feature)
1. Press `Ctrl+F` to enter search mode
2. Type your search query
3. Use arrow keys or `Ctrl+F` to navigate through matches
4. Press `Enter` to accept the current match, or `Escape` to cancel

### Search and Replace (New Feature)
1. Press `Ctrl+F` to enter search mode
2. Type your search query
3. Press `Ctrl+R` to switch to replace mode
4. Type your replacement text
5. Press `Enter` to start interactive replacement

### Interactive Replacement
Once you've entered the search query and replacement text, the editor will highlight the first match and present you with these options:

- **`y` or `Y`** - Replace the current match and move to the next one
- **`n` or `N`** - Skip the current match and move to the next one
- **`a` or `A`** - Replace all remaining matches automatically
- **`q` or `Q`** or `Escape` - Quit the replace operation

## Key Bindings
- `Ctrl+F` - Open search prompt
- `Ctrl+R` (while in search mode) - Switch to replace mode
- `Ctrl+R` (while in normal mode) - Remove current line (existing functionality)

## Implementation Details

### New Components
1. **ReplaceMode enum** - Tracks the state of the replace operation:
   - `ReplaceText` - Prompting for replacement text
   - `Interactive` - Interactive replacement mode

2. **PromptMode::Replace** - New variant added to the existing PromptMode enum:
   - Stores: search query, replacement text, saved cursor state, last match position, and current replace mode

3. **replace_current_match()** method - Handles the actual text replacement:
   - Removes the search query bytes from the current cursor position
   - Inserts the replacement text bytes
   - Updates the editor state (dirty flag, byte count, row rendering)

### Workflow
```
Normal Mode
    ↓ (Ctrl+F)
Search Mode (Find)
    ↓ (Ctrl+R)
Replace Mode (ReplaceText) - Prompt for replacement
    ↓ (Enter)
Replace Mode (Interactive) - Interactive replacement
    ↓ (y/n/a/q)
Normal Mode (operation complete)
```

## Status Messages
- **Search mode**: "Search (Use ESC/Arrows/Enter): {query}"
- **Replace text prompt**: "Replace '{search_query}' with: {replacement}"
- **Interactive mode**: "Replace this occurrence? (y/n/a=all/q=quit)"
- **Completion**: "Replace complete" or "No more matches"
- **Cancel**: "Replace cancelled"

## Example Usage

### Replace all occurrences of "hello" with "hi":
1. `Ctrl+F`
2. Type: `hello`
3. `Ctrl+R`
4. Type: `hi`
5. `Enter`
6. Press `a` to replace all

### Selective replacement:
1. `Ctrl+F`
2. Type your search query
3. `Ctrl+R`
4. Type your replacement
5. `Enter`
6. For each match, press `y` to replace or `n` to skip

## Notes
- The feature preserves the original cursor position if the operation is cancelled
- Match highlighting is updated during navigation
- The editor is marked as "dirty" after any replacement
- Byte count is accurately maintained when replacements have different lengths
- Empty search queries are not allowed and will cancel the operation

