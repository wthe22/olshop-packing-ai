// The *No groups yet* line's rule: the line is shown while the rule's only group has no filter.
// Pure logic; no window opens and nothing is read.
//
// Run in `app/`: `npm test` (Node's own test runner; the project has no test framework).

import { test } from "node:test";
import assert from "node:assert/strict";

import { unconfigured } from "../src/lib/unconfigured.ts";

test("only one group with no filter is a rule with no groups yet", () => {
  // (the groups, whether the line shows)
  const cases = [
    [[{ filter: null }], true],
    [[{ filter: 'name contains "sepatu"' }], false],
    [[{ filter: 'name contains "sepatu"' }, { filter: null }], false],
    [[], false],
  ];
  for (const [groups, expected] of cases) {
    assert.equal(unconfigured(groups), expected, JSON.stringify(groups));
  }
});
