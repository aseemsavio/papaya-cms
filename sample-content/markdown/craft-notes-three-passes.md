---
title: "Three passes: notes from the studio"
slug: "craft-notes-three-passes"
date: "2026-07-02"
author: "Aseem Savio"
summary: "Short notes on the studio loop — restraint, readable type, and the checklist we run before anything ships."
tags: ["craft", "process", "checklist"]
cover_image: "/assets/images/otia-cover-craft.svg"
---

# Three passes: notes from the studio

Most of the work is not making things. It is deciding what to leave alone.

## Readable beats beautiful

A gorgeous typeface at the wrong size is worse than a plain one at the right size. We test at
three sizes on three screens before we commit:

| Check | Target | Fails when |
| :--- | :---: | :--- |
| Line length | 60–75 characters | You have to move your head to track a line |
| Line height | 1.6–1.85 | Paragraphs read as blocks instead of sentences |
| Contrast | ≥ 4.5:1 | Text looks "atmospheric" instead of readable |

## Checklists are a form of care

- [x] Every interactive element reachable by keyboard
- [x] Every image carries meaningful `alt` text or `alt=""`
- [x] Motion respects `prefers-reduced-motion`
- [x] The page renders without JavaScript
- [ ] Print stylesheet reviewed with an actual printer in mind

> [!NOTE]
> The last item stays unchecked on purpose. It is the one we keep forgetting.

## A small unit of craft

```js
// Spend one accent, and spend it where someone can act.
const accent = "var(--sage)";
const onAction = [".code-copy:hover", ".carousel-dot[aria-current='true']", "a:focus-visible"];
```

---

If this resonates, the [theme that renders this journal](/posts/otia-design-system) is the
long form version of it.
