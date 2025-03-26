# Begin's Comp died - Restart Speedrun

- I am running my server for Titch eventsub locally! So if it crashes...I have to unsub to all my events:

## - restart on ngrok on 8080

```bash
ngrok http 8080
```

* Open the Titch Dev Console and update the Ngrok callback URL

https://dev.twitch.tv/console

You should be using a URL w/ the path /eventsub
For example: `https://f524-47-151-135-242.ngrok-free.app/eventsub`

## list all previous subs, and grab the IDs

```bash
cd code/eventsub-python-fun
./list.sh | jq -r ".data[].id"
```

There is a chance you get an error for unauthorized:

```error
./list.sh
{"error":"Unauthorized","status":401,"message":"Invalid OAuth token"}%
```

In that case check your OAUTH Token

- Take the IDs and update the unsub.sh script with the IDs

- Run the unsub.sh script

```bash
./unsub.sh
```

- Update the sub scripts NGROK URL and resub

```bash
./sub.sh
```

Run the Webserver

```bash
python main.py
```

---

## I think we have a rust one too
## Event Sub Server

```bash
cd code/eventsub-python-fun
source venv/bin/activate
```

