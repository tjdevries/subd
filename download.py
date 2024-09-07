import json
import time
import requests

base_url = 'http://localhost:3000'

def get_clip(clip_id):
    url = f"{base_url}/api/clip?id={clip_id}"
    response = requests.get(url)
    return response.text

def download_mp3(url):
    response = requests.get(url)
    if response.status_code == 200:
        timestamp = int(time.time())
        filename = f"{timestamp}.mp3"
        with open(f"ai_songs/{filename}", 'wb') as file:
            file.write(response.content)
        return filename
    return None

if __name__ == "__main__":
    # "audio_url"
    mp3_url = "https://cdn1.suno.ai/33cab0ea-7370-431c-a898-0f156ec119ae.mp3"
    downloaded_file = download_mp3(mp3_url)
    if downloaded_file:
        print(f"MP3 downloaded as: {downloaded_file}")
    else:
        print("Failed to download MP3")

    # id = "33cab0ea-7370-431c-a898-0f156ec119ae"
    # result = get_clip(id)
    # print(result)
    
    # id = "33cab0ea-7370-431c-a898-0f156ec119ae"
    # result = get_clip(id)
    # print(result)
