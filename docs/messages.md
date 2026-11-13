# Messages and command history

`get_message_list(group, page_num, page_size)` reads one page from the account's
`Alarm`, `Command`, or `News` group. `get_alarm_list`, `get_command_list`, and
`get_news_list` are convenience methods. Page numbers start at 1; page number and
size must be positive. The caller chooses when to request another page.
`get_unread_messages_count()` reads account counts. None of these calls marks,
deletes, polls, or refreshes anything.

Both reads return `Option<MessageResp>` because the current API allows an absent
result. Missing/null fields remain `None`; an empty `messages` array stays empty.
Counts and unknown fields are preserved. `MessageId` preserves string IDs
(including leading zeros) and numeric IDs without floating-point conversion.
`create_time` is Unix milliseconds; `message_time` retains the server's raw local
time string without inventing a timezone or a timestamp for missing values.
Unknown `read_status` codes remain integers. Debug output redacts message data.

`update_message_status` accepts a typed `MessageAction`:

- `Read(id)` sends `READ` for that ID.
- `Delete(id)` sends `DELETE` for that ID.
- `DeleteAll(group)` sends `DELETE_ALARM`, `DELETE_COMMAND`, or `DELETE_NEWS`.

Each update is one PUT to `/message/status`. It returns `CommandOutcome`; check
its state for completion, rejection, invalid session, or an unknown outcome.
No operation retries or fetches vehicle/charging status, including on failure.
An unknown outcome must not be blindly replayed.

CLI equivalents:

```text
messages alarm 1 20
alarm-list 1 20
command-list 2 10
news-list 1 20
messages-unread
message-status read "001-synthetic"
message-status delete 42
message-status delete-all news
```

IDs in the CLI use JSON string or integer syntax. Bulk deletion requires both
`delete-all` and an explicit group; there is no default group or wildcard.
The CLI prints redacted results, as with other account reads.

Protocol source: client-ng revision `5ffd4e2bbb501d1afbfb272d1f546c0dd2030d0e`,
`api/message/__init__.py` and `schema.py`. The local gateway's message handler
starts pagination at 1; its polling and mark/delete policies belong to the caller.
Synthetic fixtures exercise each group and action, pagination, exact IDs, empty
results, rejection and failure. No production validation has been performed.
