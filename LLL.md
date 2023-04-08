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

#### RESERVED (1 << 11)

Reserved for a future use.

#### RESERVED (1 << 12)

Reserved for a future use.

#### RESERVED (1 << 13)

Reserved for a future use.

#### RESERVED (1 << 14)

Reserved for a future use.

#### RESERVED (1 << 15)

Reserved for a future use.

#### ADJECTIVE_ORDER_BLOCK (1 << 16, 1 << 17, 1 << 18, 1 << 19)

These four bits are used, not as flags but as a 4 bit number indicating
which of some exclusive catagories the lexeme belongs to. These values
are denoted below with decimal numbers from 0 to 15 inclusive. It is
expected that many users of these 4 bits will mask off the other bits,
and index or shift as necessary to create a value directly corresponding
to the given numbers.

The numeric value of these categories indicates a relative ordering of
how adjectives are used in English: Adjectives in categories of lower
values are expected to occur before those in higher ones within
adjectival phrases. For example one may say of a cow that is both big
(category 6), and brown (category 10) that it is "a big brown cow". To
say it is "a brown big cow" sounds strange/incorrect to native English
speakers.

It should be noted that it is possible written rules, like these ones
about adjctive order, to actually be born out in reality at the time of
their creation. Also, languages can and do change over time. So, it is
possible that this ordering may be incorrect.

We collectively refer to these categories as the adjective order block
categories, even though some (currently reserved) categories may be
assigned to non-categories.

##### ADJECTIVE_ORDER_BLOCK_NONE (0)

The lexeme is not a member of any of the adjective order categories
described in this version of this document.

We reserve the ability to add new categories to any remaining reserved
category sections, including categories indicating non-adjectives, so
this value cannot be relied on to indicate the lexeme is not an
adjective.

##### RESERVED (1)

Reserved for a future use.

##### RESERVED (2)

Reserved for a future use.

##### RESERVED (3)

Reserved for a future use.

##### ADJECTIVE_ORDER_BLOCK_QUANTITY (4)

The lexeme is a quantity adjective.
Examples: one, three, some

##### ADJECTIVE_ORDER_BLOCK_OBSERVATION (5)

The lexeme is an opinion/observation adjective.
Examples: good, bad, clever

##### ADJECTIVE_ORDER_BLOCK_SIZE (6)

The lexeme is a size adjective.
Examples: large, medium-sized, small

##### ADJECTIVE_ORDER_BLOCK_PHYSICAL (7)

The lexeme is a physical quality adjective.
Examples: soft, lumpy, cluttered

##### ADJECTIVE_ORDER_BLOCK_SHAPE (8)

The lexeme is a shape adjective.
Examples: square, spherical, helical

##### ADJECTIVE_ORDER_BLOCK_AGE (9)

The lexeme is a age adjective.
Examples: young, old, new

##### ADJECTIVE_ORDER_BLOCK_COLOUR (10)

The lexeme is a colour adjective.
Examples: red, green, blue

##### ADJECTIVE_ORDER_BLOCK_ORIGIN (11)

The lexeme is a colour adjective.
Examples: Terran, Martian, Venusian

##### ADJECTIVE_ORDER_BLOCK_MATERIAL (12)

The lexeme is a material adjective.
Examples: metal, plastic, copper

##### RESERVED (13)

Reserved for a future use.

##### RESERVED (14)

Reserved for a future use.

##### RESERVED (15)

Reserved for a future use.

### Lexeme

The lexeme is encoded in UTF-8, and starts after the last flags byte and
continues until the end of the block, ending at the last byte of the
block.
