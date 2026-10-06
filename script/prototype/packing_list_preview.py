"""THROWAWAY PROTOTYPE of the A4 packing list. Reference for script/packing/packing_list.py only.

Not production code: reads samples/ directly, categories are hard-coded lambdas instead of
categories.toml, group tie-break differs from the spec (01-requirements, rule 6), no `pick`
layout. What it does show correctly: page header, category bars, group rows, tracking-ID
columns with page continuation, two-column pick summary with wrapping names, fonts, sizes.

Layouts: `summary` and `tracking` (= `full` in the spec). `demo` builds made-up multi-item
orders (fake order and tracking IDs) to show multi-item groups.

usage (needs fpdf2):  python packing_list_preview.py <1|2|demo> <summary|tracking> <out.pdf>
"""
import csv, collections, sys
from fpdf import FPDF

S = 'D:/workspace/software/olshop-packing-ai/samples/'
F = 'C:/Windows/Fonts/'
OK = ('Perlu dikirim', 'Menunggu pengambilan')


def load(fn):
    with open(S + fn, encoding='utf-8-sig', newline='') as f:
        rows = [{k: v.strip('\t').strip() for k, v in r.items()} for r in csv.DictReader(f)]
    orders = collections.defaultdict(list)
    for r in rows:
        if (r['Order Status'], r['Order Substatus']) == OK:
            orders[r['Order ID']].append(r)
    return orders


def disp(r):
    n = r['Product Name'].split('|')[0].strip()
    v = r['Variation'].strip()
    return n if v in ('', 'Default') else f'{n} — {v}'


def has(o, w, col='Product Name'):
    return any(w in r[col].lower() for r in o)


CATS = [
    ('A', 'Sepatu', lambda o: has(o, 'sepatu')),
    ('B', 'Spion & Knalpot', lambda o: not has(o, 'sepatu') and (has(o, 'spion') or has(o, 'knalpot'))),
    ('C', 'Motor lainnya', lambda o: has(o, 'sepeda motor', 'Product Category')),
    ('Z', 'Lainnya', lambda o: True),
]


def cat(o):
    return next(c for c in CATS if c[2](o))


def sig(o):
    return tuple(sorted((r['SKU ID'], int(r['Quantity'])) for r in o))


which, layout, out = sys.argv[1], sys.argv[2], sys.argv[3]
b1, b2 = load('orders-06-1.csv'), load('orders-06-2.csv')
names = {r['SKU ID']: disp(r) for o in b2.values() for r in o}
if which == '1':
    batch, batch_no, printed = dict(b1), '1', '14:30'
elif which == '2':
    batch, batch_no, printed = {k: v for k, v in b2.items() if k not in b1}, '2', '16:40'
else:
    # SYNTHETIC demo: real SKUs, made-up orders and tracking IDs, to show multi-item groups.
    import random
    random.seed(7)
    sk = {names[k]: k for k in names}
    pick = lambda s: next(k for n, k in sk.items() if n.startswith(s))
    combos = [
        ([('Sepatu Standar Samping Motor', 1), ('Spion Beat — Standard, honda', 1)], 9),
        ([('Spion Beat — Standard, honda', 1), ('Spion Beat — Mini, honda', 1)], 4),
        ([('Cover Knalpot Beat — FI 2012-2015', 1), ('Spion Beat — Chrome Standard, honda', 1)], 3),
        ([('Spion PCX — Polos Pendek, Honda', 2), ('Spion Yamaha X1 — GG TEKUK, YAMAHA', 1),
          ('Hook/ Cantelan luggage universal MPX', 1)], 2),
        ([('Sepatu Standar Samping Motor', 2), ('Diamond Powder — W7', 1)], 1),
    ]
    batch, n = {}, 0
    for items, cnt in combos:
        for _ in range(cnt):
            n += 1
            oid = f'DEMO{n:04d}'
            trk = 'JY' + ''.join(random.choice('0123456789') for _ in range(10))
            batch[oid] = [dict(next(r for o in b2.values() for r in o if r['SKU ID'] == pick(s)),
                               **{'Quantity': str(q), 'Order ID': oid, 'Tracking ID': trk,
                                  'RTS Time': '06/10/2026 17:00:00'}) for s, q in items]
    batch_no, printed = '3 (DEMO — synthetic orders)', '17:10'

cnt = collections.Counter(sig(o) for o in batch.values())
members = collections.defaultdict(list)
for o in batch.values():
    members[sig(o)].append(o)
for s in members:
    members[s].sort(key=lambda o: (o[0]['RTS Time'][6:10] + o[0]['RTS Time'][3:5] + o[0]['RTS Time'][:2]
                                   + o[0]['RTS Time'][11:], o[0]['Order ID']))
first = {s: v[0] for s, v in members.items()}
units_total = sum(int(r['Quantity']) for o in batch.values() for r in o)
SUB = f'{len(batch)} orders · {units_total} units · {len(cnt)} groups · printed {printed}'
title = f'Packing list · 2026-10-06 · Batch {batch_no}'


class P(FPDF):
    def header(self):
        self.set_font('A', 'B', 13)
        self.cell(150, 7, title)
        self.set_font('A', '', 9.5)
        self.cell(40, 7, f'page {self.page_no()}/{{nb}}', align='R', new_x='LMARGIN', new_y='NEXT')
        self.cell(0, 5, SUB, new_x='LMARGIN', new_y='NEXT')
        self.ln(1)


pdf = P('P', 'mm', 'A4')
pdf.set_margins(10, 10, 10)
pdf.set_auto_page_break(False)
pdf.add_font('A', '', F + 'arial.ttf'); pdf.add_font('A', 'B', F + 'arialbd.ttf')
pdf.add_font('M', '', F + 'consola.ttf')
pdf.add_page()
BOTTOM = pdf.h - 10
LH, TH, NCOL, CW, X_ITEMS = 5.2, 4.6, 6, 29.6, 22


def need(h, cont=None):
    if pdf.get_y() + h > BOTTOM:
        pdf.add_page()
        if cont:
            pdf.set_font('A', '', 9.5); pdf.set_text_color(90)
            pdf.cell(0, 5, cont, new_x='LMARGIN', new_y='NEXT'); pdf.set_text_color(0)
        return True
    return False


seq = 0
for c in CATS:
    sigs = sorted((s for s in cnt if cat(first[s])[0] == c[0]), key=lambda s: (-cnt[s], str(s)))
    if not sigs:
        continue
    n_ord = sum(cnt[s] for s in sigs)
    n_units = sum(cnt[s] * sum(q for _, q in s) for s in sigs)
    need(6.5 + LH * 2)
    pdf.set_fill_color(225, 225, 225)
    pdf.set_font('A', 'B', 11)
    pdf.cell(150, 6.5, f' {c[0]}  {c[1]}', fill=True)
    pdf.set_font('A', '', 9.5)
    pdf.cell(40, 6.5, f'{n_ord} orders · {n_units} units', fill=True, align='R',
             new_x='LMARGIN', new_y='NEXT')
    for s in sigs:
        seq += 1
        items = sorted(f'{names[k]}  ×{q}' for k, q in s)
        need(LH * len(items) + (TH + 1 if layout == 'tracking' else 0) + 1.2)
        y = pdf.get_y()
        pdf.set_font('A', 'B', 11.5); pdf.cell(12, LH, f'{seq:02d}')
        pdf.set_font('A', '', 10.5)
        for i, t in enumerate(items):
            pdf.set_xy(X_ITEMS, y + LH * i); pdf.cell(158, LH, t)
        pdf.set_xy(180, y); pdf.set_font('A', 'B', 11.5)
        pdf.cell(20, LH, f'{cnt[s]}', align='R')
        pdf.set_xy(10, y + LH * len(items))
        if layout == 'tracking':
            trk = [o[0]['Tracking ID'] for o in members[s]]
            pdf.set_y(pdf.get_y() + 0.6)
            pdf.set_font('M', '', 10)
            label = f'{seq:02d} (continued)  ' + '  /  '.join(items)
            for r0 in range(0, len(trk), NCOL):
                if need(TH, label):
                    pdf.set_font('M', '', 10)
                yy = pdf.get_y()
                for j, t in enumerate(trk[r0:r0 + NCOL]):
                    pdf.set_xy(X_ITEMS + j * CW, yy); pdf.cell(CW, TH, t)
                pdf.set_xy(10, yy + TH)
        yb = pdf.get_y() + 0.6
        pdf.set_draw_color(170, 170, 170)
        pdf.line(10, yb, 200, yb)
        pdf.set_xy(10, yb + 0.6)
    pdf.ln(1.5)

# pick summary (two columns, units first, names wrap)
units = collections.Counter()
for o in batch.values():
    for r in o:
        units[r['SKU ID']] += int(r['Quantity'])
lst = sorted(units.items(), key=lambda x: names[x[0]])
half = (len(lst) + 1) // 2
need(6.5 + LH * (half + 4))
pdf.ln(1)
pdf.set_font('A', 'B', 11); pdf.set_fill_color(225, 225, 225)
pdf.cell(190, 6.5, ' Pick summary (units to take from stock)', fill=True, new_x='LMARGIN', new_y='NEXT')
y0 = pdf.get_y()
for col, part in enumerate((lst[:half], lst[half:])):
    pdf.set_xy(10 + col * 96, y0)
    for k, u in part:
        x, y = pdf.get_x(), pdf.get_y()
        pdf.set_font('A', '', 10)
        lines = pdf.multi_cell(80, 5.0, names[k], dry_run=True, output='LINES')
        pdf.set_font('A', 'B', 10.5); pdf.cell(12, 5.0, str(u), align='R')
        pdf.set_font('A', '', 10)
        for i, ln in enumerate(lines):
            pdf.set_xy(x + 14, y + 5.0 * i); pdf.cell(80, 5.0, ln)
        pdf.set_xy(x, y + 5.0 * len(lines) + 0.4)

pdf.output(out)
print(out, pdf.pages_count, 'pages', len(cnt), 'groups', len(batch), 'orders')
