# import fal_client
#
# handler = fal_client.submit(
#     "fal-ai/sadtalker",
#     arguments={
#         "source_image_url": "https://storage.googleapis.com/falserverless/model_tests/sadtalker/anime_girl.png",
#         "driven_audio_url": "https://storage.googleapis.com/falserverless/model_tests/sadtalker/deyu.wav"
#     },
# )
#
# result = handler.get()

import fal_client
import base64
import mimetypes

# These files are tooo big!
# I'd have to chunk
# Replace with your local file paths
image_file_path = 'prime.jpg'
# audio_file_path = 'ff5608b5-055a-434a-a380-436e09436c9f.mp3'
audio_file_path = 'TwitchChatTTSRecordings/1701059381_beginbot_prime.wav'

# Get the MIME type of the image file
image_mime_type, _ = mimetypes.guess_type(image_file_path)
if image_mime_type is None:
    image_mime_type = 'application/octet-stream'  # Default fallback

# Read and encode the local image file to Base64 data URI
with open(image_file_path, 'rb') as image_file:
    image_data = image_file.read()
    encoded_image = base64.b64encode(image_data).decode('utf-8')
    image_data_uri = f"data:{image_mime_type};base64,{encoded_image}"

# Get the MIME type of the audio file
audio_mime_type, _ = mimetypes.guess_type(audio_file_path)
if audio_mime_type is None:
    audio_mime_type = 'application/octet-stream'  # Default fallback

# Read and encode the local audio file to Base64 data URI
with open(audio_file_path, 'rb') as audio_file:
    audio_data = audio_file.read()
    encoded_audio = base64.b64encode(audio_data).decode('utf-8')
    audio_data_uri = f"data:{audio_mime_type};base64,{encoded_audio}"

handler = fal_client.submit(
    "fal-ai/sadtalker",
    arguments={
        "source_image_url": image_data_uri,
        "driven_audio_url": audio_data_uri
    },
)

result = handler.get()
print(result)
