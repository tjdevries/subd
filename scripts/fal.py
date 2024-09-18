import fal_client
 
handler = fal_client.submit(
    "fal-ai/lora",
    arguments={
        "model_name": "stabilityai/stable-diffusion-xl-base-1.0",
        "prompt": "photo of a rhino dressed suit and tie sitting at a table in a bar with a bar stools, award winning photography, Elke vogelsang"
    },
)
 
result = handler.get()
print(result)
