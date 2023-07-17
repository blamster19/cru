# cru - crude record utility

cru is a cli tool for keeping plaintext notes with built-in git integration (partially implemented). I created this tool because I wanted easy to use notetaking app which could be accessed from terminal and allowed for easy synchronization. I figured remote git repository would be ideal for this task as it provides version control out of the box and doesn't require third-party cloud storage providers (everyone uses git hosting already, duh).

## Usage

To access help just type
```
$ cru
```
or
```
$ cru help
```
cru suppots following commands:
- `new [name]` `n [name]` - create new note (record); `[name]` is optional - if not present, timestamp will be used as filename
- `ls` `l` - list all notes
- `edit [name]` `e [name]` - edit note
- `show [name]` `s [name]` - show note

## TODO

- git remote integration
- repository encryption
- record removal
- shell completions

