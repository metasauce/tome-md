---
title: Weekly Team Meeting
date: 12 March 2026
attendees: Jane Doe, John Smith, Lisa Park
---

# Agenda

1. Welcome and project updates
2. Launch readiness
3. Budget and next steps

---

## Project updates

The _website redesign_ is on track for **15 April**. We will ~~move the launch to May~~ keep the original date and review progress on the 15<sup>th</sup> April.

### Action items

- [x] Share the meeting notes
- [x] Book the follow-up room
- [ ] Confirm the QA schedule
  - Smoke tests
  - Accessibility review
- [ ] Send the updated timeline

> **Decision:** Keep the launch date. CO<sub>2</sub> reporting is due on the same day.

## Launch readiness

The release command is `./scripts/release.sh --dry-run`. Run it before the final review:

```bash
./scripts/release.sh --dry-run
./scripts/release.sh
```

**Reminder:** Review the draft before Friday.\
Bring comments to the meeting.

## Budget

| Item    | Owner |   Status    |   Cost |
| :------ | :---- | :---------: | -----: |
| Design  | Jane  |    Done     | €2,400 |
| Content | John  | In progress | €1,100 |
| QA      | Lisa  |   Planned   |   €900 |

See the [project board](https://example.com/board) for details.
