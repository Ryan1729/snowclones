# The Labelled Lexeme List (.lll) Format

The Labelled Lexeme List format is a binary file format for storing
lexemes, (words or groups of words that form a unit) along with some
metadata about each lexeme.

The format consists of a magic number followed immediately by some
number of blocks, until the end of the file.

## Magic Number

The magic number is a 4 byte value at the very beginning of the file
which indicates the file type and the version number.

The first three bytes each are 108, AKA 0x6C, AKA ASCII lowercase l.
The fourth byte is the version number. That is, a value of zero, (all
bits zero) corresponds to version 0. In contrast a version of 48 would
corrspond to version 48, given that 49 different versions are deemed
necessary. (Note that 48 is the value of the ASCII value of the numeral
"0" is 48.)

## Block Format (Version 0)

| Length |  FEF  |  Flags  |      Lexeme      |
| 7 bits | 1 bit | 3 bytes | Length - 4 bytes |

The first byte of the first block follows immediately after the end of
the last byte Magic Number. The next block follows immediately after the
previous block, and the next after that, and so on until the end of the
file.

### Length

The length is an 8 bit unsigned integer indicating the length of the
block in bytes, including the flags and one byte for the length itself.
Note this implies that the length can never be less than four in a
valid file.

### Future Expansion Flag (FEF)

This bit is reserved for future expansion. While the details of this
expansion are not fully worked out, we can promise that the length value
(with this bit masked off) will still be correct. Given that the value
is 0, then only what is written in this version of this document
applies. If the value is 1 then readers of the binary file which do not
know the exact, currently undecided semantics of this flag must skip
over that block using the length in order to correctly process the rest
of the file.

### Flags

The flags as a collection are 3 bytes long. Each bit of those three
bytes are referred to individually as a flag. The exact semantics of
each of the flags will be specified on an as needed basis. This means
that if the semantics of a given flag has not been decided yet, a reader
cannot assume anything about the value of that flag, or the meaning
thereof.

The following list of flags starts at the least significant bit of the
first flags byte, and continues upward, going though the other bits of
that byte, then from the most significant bit of the first flags byte to
the least significant bit of the second flags byte and so on.

#### SINGULAR_NOUN (1 << 0)

Indicates that the lexeme is a singular noun.
Examples: Fork, Sheep

#### PLURAL_NOUN (1 << 1)

Indicates that the lexeme is a plural noun.
Examples: Forks, Sheep

#### MASS_NOUN (1 << 2)

Indicates that the lexeme is a mass noun.
Examples: milk, cabbage

#### RESERVED (1 << 3)

Reserved for a future use, probably relating to nouns.

#### INTRANSITIVE_VERB (1 << 4)

Indicates that the lexeme is a verb that does not need a direct object.
Examples: flow, understand

#### TRANSITIVE_VERB (1 << 5)

Indicates that the lexeme is a verb that can be used with a direct object.
Examples: have, understand

#### RESERVED (1 << 6)

Reserved for a future use, probably relating to verbs.

#### RESERVED (1 << 7)

Reserved for a future use, probably relating to verbs.

#### THIRD_PERSON_SINGULAR_VERB (1 << 8)

Indicates that the lexeme is a verb in third person singular form.
Examples: runs, tries

#### RESERVED (1 << 9)

Reserved for a future use, probably relating to verbs.

#### FIRST_PERSON_SINGULAR_VERB (1 << 10)

Indicates that the lexeme is a verb in first person singular form.
Examples: run, try

### Lexeme

The lexeme is encoded in UTF-8, and starts after the last flags byte and
continues until the end of the block, ending at the last byte of the
block.
