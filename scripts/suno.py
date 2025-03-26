from tqdm import tqdm
import json
import time
import requests

# replace your vercel domain
base_url = 'http://localhost:3000'


def custom_generate_audio(payload):
    url = f"{base_url}/api/custom_generate"
    response = requests.post(url, json=payload, headers={'Content-Type': 'application/json'})
    return response.json()


def extend_audio(payload):
    url = f"{base_url}/api/extend_audio"
    response = requests.post(url, json=payload, headers={'Content-Type': 'application/json'})
    return response.json()

def generate_audio_by_prompt(payload):
    url = f"{base_url}/api/generate"
    response = requests.post(url, json=payload, headers={'Content-Type': 'application/json'})
    return response.json()


def get_audio_information(audio_ids):
    url = f"{base_url}/api/get?ids={audio_ids}"
    response = requests.get(url)
    j = response.json()

    return j


def get_quota_information():
    url = f"{base_url}/api/get_limit"
    response = requests.get(url)
    return response.json()

def get_clip(clip_id):
    url = f"{base_url}/api/clip?id={clip_id}"
    response = requests.get(url)
    return response.json()

def generate_whole_song(clip_id):
    payloyd = {"clip_id": clip_id}
    url = f"{base_url}/api/concat"
    response = requests.post(url, json=payload)
    return response.json()


if __name__ == '__main__':
    # This returns the song itself, not the json
    # url = "https://audiopipe.suno.ai/?item_id=01188353-75b0-43c2-8bb2-612d847a7129"
    # print(url)
    # with open("song4.json", "wb") as handle:
    #     r = requests.get(url, stream=True)
    #     print("Status:\n")
    #     print(r.status_code)
    #     for data in tqdm(r.iter_content()):
    #         print(data)
    #         handle.write(data)
    # exit(1)

    # with open("result.json", "r") as file:
    #     data = json.load(file)
    #     url = data[0]['audio_url']
    #     print(url)
    #
    #     # exit(1)

    data = generate_audio_by_prompt({
        "prompt": "A popular song about not letting friends use Arch Linux, so they will be happy, sung by a deep-voiced male singer, fast and lively. The lyrics depict the sorrow of people using Arch Linux.",
        "make_instrumental": False,
        "wait_audio": False
    })

    ids = f"{data[0]['id']},{data[1]['id']}"
    print(f"ids: {ids}")

    for _ in range(60):
        data = get_audio_information(ids)
        if data[0]["status"] == 'streaming':
            print(f"{data[0]['id']} ==> {data[0]['audio_url']}")
            print(f"{data[1]['id']} ==> {data[1]['audio_url']}")
            break
        # sleep 5s
        time.sleep(5)
    # test_prompt = {
    #   "prompt": "[Verse 1]\nCruel flames of war engulf this land\nBattlefields filled with death and dread\nInnocent souls in darkness, they rest\nMy heart trembles in this silent test\n\n[Verse 2]\nPeople weep for loved ones lost\nBattered bodies bear the cost\nSeeking peace and hope once known\nOur grief transforms to hearts of stone\n\n[Chorus]\nSilent battlegrounds, no birds' song\nShadows of war, where we don't belong\nMay flowers of peace bloom in this place\nLet's guard this precious dream with grace\n\n[Bridge]\nThrough the ashes, we will rise\nHand in hand, towards peaceful skies\nNo more sorrow, no more pain\nTogether, we'll break these chains\n\n[Chorus]\nSilent battlegrounds, no birds' song\nShadows of war, where we don't belong\nMay flowers of peace bloom in this place\nLet's guard this precious dream with grace\n\n[Outro]\nIn unity, our strength will grow\nA brighter future, we'll soon know\nFrom the ruins, hope will spring\nA new dawn, we'll together bring",
    #   "tags": "pop metal male melancholic",
    #   "title": "Silent Battlefield",
    #   "make_instrumental": False,
    #   "wait_audio": False
    # }
    # ids = f"{data[0]['id']},{data[1]['id']}"
    # print(f"ids: {ids}")
    #
    # for _ in range(60):
    #     data = get_audio_information(ids)
    #     if data[0]["status"] == 'streaming':
    #
    #         # We need to time out and retry
    #         url = data[0]['audio_url']
    #         print(url)
    #         with open("song3.json", "wb") as handle:
    #             r = requests.get(url, stream=True)
    #             for data in tqdm(r.iter_content()):
    #                 handle.write(data)
    #
    #         print(f"{data[0]['id']} ==> {data[0]['audio_url']}")
    #         print(f"{data[1]['id']} ==> {data[1]['audio_url']}")
    #         break
    #     # sleep 5s
    #     time.sleep(5)
