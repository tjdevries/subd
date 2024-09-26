import os
import requests

payload = {}

response = requests.post(
    "https://api.gooey.ai/v2/Lipsync",
    headers={
        "Authorization": "bearer " + os.environ["GOOEY_API_KEY"],
    },
    json=payload,
)
assert response.ok, response.content

result = response.json()
print(response.status_code, result)
