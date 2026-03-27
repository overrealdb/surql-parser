-- Creates an analyzer suitable for source code analysis.
DEFINE ANALYZER code TOKENIZERS class,camel FILTERS lowercase,ascii;
