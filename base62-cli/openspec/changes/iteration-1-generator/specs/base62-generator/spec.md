## Purpose

Provides a compact, URL‑safe Base62 representation for arbitrary binary data, enabling encoding and decoding of hashes for identifiers, filenames, and short URLs.

## ADDED Requirements

### Requirement: Base62 Encode
The system SHALL allow users to encode any binary input into a Base62 string.

#### Scenario: Successful encode
- **WHEN** the user provides a byte array (e.g., a SHA‑256 hash) to the `encode` function
- **THEN** the function returns a Base62 string containing only characters `0‑9`, `A‑Z`, `a‑z` with no padding.

### Requirement: Base62 Decode
The system SHALL allow users to decode a Base62 string back into the original binary data.

#### Scenario: Successful decode
- **WHEN** the user provides a valid Base62 string to the `decode` function
- **THEN** the function returns the original byte array that was encoded.

### Requirement: Invalid Input Handling
The system SHALL return an error when decoding an invalid Base62 string (containing characters outside the allowed alphabet).

#### Scenario: Decode with invalid characters
- **WHEN** the user attempts to decode the string "!invalid$"
- **THEN** the function returns an error indicating invalid characters.
