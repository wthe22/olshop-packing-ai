# Search

Finds saved orders of the day by the fields the rules can use ([inputs](../app/inputs.md)): Order ID, tracking ID, courier, product name, variation, Seller SKU, line quantity, total quantity and different items. **Search** in the window frame opens it over the current screen, as Categories does; **Close** returns to it. It only reads: nothing is changed or saved.

- **The conditions** are made with the same boxes as [Categories](categories.md): a group *All of these* / *Any of these*, rows of [field ▾] [operator ▾] [value] [not] [×] and [+ condition] [+ group]. The field list is the one above; the operators are the ones [Categories](categories.md) offers, *contains* first for a text field. Matching ignores case. An item field matches an order when one of its lines matches, as in the rules file ([the condition language](../rules-file/conditions.md)).
- **Scope**: *This day* (the day in the title bar; the default) or *All days* ([Q-UI-3](../questions/ui.md)).
- **Results** show the number of orders and batches, then one row per order: Order ID · tracking ID · courier · items (name × quantity) · batch · saved PDF. In *All days* each row also shows its day. **Open** opens that saved PDF in the PC's PDF viewer, as on the Day screen.
- **Saved orders only.** Orders of an open draft are not searched: they have no saved PDF yet.
- **Orders saved before search existed** keep only their IDs ([Q-UI-3](../questions/ui.md)). Their row says *items not stored*, and they are found by Order ID or tracking ID only.
- **No match**: "No saved order matches."
