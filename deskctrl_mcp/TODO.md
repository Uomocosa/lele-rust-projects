Dont really like the way we are sending messages to telegram could we remove the send report tool and instead send each time a message to telegram (with or without an image)
I would like
1. ~~"📋 Starting Session - YYYY_MM_DD \[hh:mm:ss\]" when we start a new mcp session.~~ REVERTED 2026-08-14: the banner fired on every PC boot (the agent host launches the MCP server), so it was removed. Do not re-add.
2. Look at the tools we have if possible I would like to integrate the message sending to the tools. without having a massive list of messages whenever we run the tools. I would also like a "template" for the messages like "clicking 'Increment button', expected in the image: freenet cliker state now at 8", so I can get a good understaing of what I see in the embeded image. 
3. Add a record_video tool. ~~possibly at the start of the session. and sent at session end.~~ here maybe with a summary of what we did. (Really useful for the animation we added)
   - The tool stays. The auto start-at-session-start / send-at-session-end part was REVERTED 2026-08-14: it recorded 10 minutes of idle desktop on every boot and pushed it to Telegram. Recording is now explicit-only. Do not re-add.
