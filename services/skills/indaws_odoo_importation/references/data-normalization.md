# Pre-Import Data Normalization

## Why Normalize?

Data from external files usually has:
- **Typographical errors**: "Graná" instead of "Granada"
- **Format variations**: "España", "Spain"
- **Inconsistencies**: "Madrid" vs "MADRID" vs "madrid"
- **Missing or incomplete data**

## Text Normalization Function

```python
import unicodedata

def normalize_text(text):
    """
    Normalizes text for comparison:
    - Lowercase
    - No accents
    - No extra spaces
    """
    if pd.isna(text) or not text:
        return ""

    text = str(text).lower().strip()

    # Remove accents
    text = unicodedata.normalize('NFD', text)
    text = ''.join(
        char for char in text
        if unicodedata.category(char) != 'Mn'
    )

    return text
```

## Fuzzy Matching

```python
from difflib import get_close_matches

def find_best_match(value, candidates, threshold=0.6):
    """Finds the best match using text similarity"""
    if not value or pd.isna(value):
        return None

    normalized_value = normalize_text(value)

    # Create dictionary of normalized candidates
    normalized_candidates = {}
    for candidate in candidates:
        name = normalize_text(candidate.get('name', ''))
        if name:
            normalized_candidates[name] = candidate

    # Exact search first
    if normalized_value in normalized_candidates:
        return normalized_candidates[normalized_value]

    # Fuzzy search
    matches = get_close_matches(
        normalized_value,
        normalized_candidates.keys(),
        n=1,
        cutoff=threshold
    )

    if matches:
        return normalized_candidates[matches[0]]

    return None
```
