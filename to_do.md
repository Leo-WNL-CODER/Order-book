to implement----

→ EngineRequest enum
    enum EngineRequest{
        Place(UserPayload),
        Cancel(u64),
        Modify
    }

→ Place / Cancel / Modify
→ Proper response events
→ Market-data broadcast
→ Better error handling