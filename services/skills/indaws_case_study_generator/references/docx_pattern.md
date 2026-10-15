# Word Comment Injection Pattern

python-docx has no native API for Word comments. Use this zip/XML manipulation pattern.

## Full working example

```python
import zipfile, shutil
from docx import Document
from docx.oxml.ns import qn
from docx.oxml import OxmlElement
from lxml import etree

# --- Registry ---
_comments_data = []
_comment_counter = [0]

def next_id():
    _comment_counter[0] += 1
    return _comment_counter[0]

def add_comment(para, comment_text, author="Indaws AI", date="2026-01-01T00:00:00Z"):
    """Attach a Word comment to a paragraph object."""
    cid = next_id()
    _comments_data.append((cid, comment_text, author, date))
    p = para._p

    cs = OxmlElement('w:commentRangeStart')
    cs.set(qn('w:id'), str(cid))

    ce = OxmlElement('w:commentRangeEnd')
    ce.set(qn('w:id'), str(cid))

    rr = OxmlElement('w:r')
    rpr = OxmlElement('w:rPr')
    rs = OxmlElement('w:rStyle')
    rs.set(qn('w:val'), 'CommentReference')
    rpr.append(rs)
    rr.append(rpr)
    ref = OxmlElement('w:commentReference')
    ref.set(qn('w:id'), str(cid))
    rr.append(ref)

    p.insert(0, cs)
    p.append(ce)
    p.append(rr)

def build_comments_xml():
    W = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"
    root = etree.Element(f'{{{W}}}comments', nsmap={'w': W})
    for cid, text, author, date in _comments_data:
        c = etree.SubElement(root, f'{{{W}}}comment')
        c.set(f'{{{W}}}id', str(cid))
        c.set(f'{{{W}}}author', author)
        c.set(f'{{{W}}}date', date)
        cp = etree.SubElement(c, f'{{{W}}}p')
        cr = etree.SubElement(cp, f'{{{W}}}r')
        ct = etree.SubElement(cr, f'{{{W}}}t')
        ct.text = text
    return etree.tostring(root, xml_declaration=True, encoding='UTF-8', standalone=True)

def inject_into_docx(src_path, dst_path):
    """Post-save: inject comments.xml into the DOCX zip."""
    comments_bytes = build_comments_xml()
    shutil.copy(src_path, dst_path)

    with zipfile.ZipFile(dst_path, 'r') as z:
        files = {n: z.read(n) for n in z.namelist()}

    files['word/comments.xml'] = comments_bytes

    COMMENT_REL = 'http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments'
    rels = files['word/_rels/document.xml.rels'].decode('utf-8')
    if 'rIdComments' not in rels:
        rel_line = f'<Relationship Id="rIdComments" Type="{COMMENT_REL}" Target="comments.xml"/>'
        rels = rels.replace('</Relationships>', rel_line + '</Relationships>')
        files['word/_rels/document.xml.rels'] = rels.encode('utf-8')

    COMMENT_CT = 'application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml'
    ct = files['[Content_Types].xml'].decode('utf-8')
    if 'comments.xml' not in ct:
        ct_line = f'<Override PartName="/word/comments.xml" ContentType="{COMMENT_CT}"/>'
        ct = ct.replace('</Types>', ct_line + '</Types>')
        files['[Content_Types].xml'] = ct.encode('utf-8')

    with zipfile.ZipFile(dst_path, 'w', zipfile.ZIP_DEFLATED) as z:
        for name, data in files.items():
            z.writestr(name, data)

# --- Usage ---
doc = Document()
p = doc.add_paragraph("Sector: Manufacturing")
add_comment(p, "FILLED from Odoo: category_id tags on partner 467 include 'Cliente Actual', 'Cliente Enterprise'.")

p2 = doc.add_paragraph("Revenue grew X% since Odoo go-live.")
add_comment(p2, "GAP — Revenue metric needed: Ask Jorge: 'Can you share any revenue or order volume figures before vs. after Odoo? Even a rough percentage would work.'")

tmp = "/tmp/cs_tmp.docx"
doc.save(tmp)
inject_into_docx(tmp, "/path/to/output.docx")
```

## Critical notes

- Always call `doc.save(tmp)` FIRST, then `inject_into_docx(tmp, final_path)` — inject works on an already-saved file.
- The `_comments_data` list and counter are module-level — reset them at the start of each run if reusing the module.
- `add_comment` inserts `commentRangeStart` at position 0 of the paragraph XML and appends `commentRangeEnd` + `commentReference` at the end — this wraps the entire paragraph text in the comment anchor.
- Do NOT use `etree.Element` with `set('xmlns:w', ...)` — lxml raises `ValueError: Invalid attribute name`. Use `nsmap` in the element constructor instead (as shown in `build_comments_xml`).
