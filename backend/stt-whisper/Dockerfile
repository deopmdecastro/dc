FROM onerahmet/openai-whisper-asr-webservice:latest
# Imagem base já expõe /asr em :9000. Este Dockerfile existe para permitir
# customizações locais (ex.: modelos pré-baixados, PT-BR fine-tuned).
ENV ASR_MODEL=small
ENV ASR_ENGINE=openai_whisper
