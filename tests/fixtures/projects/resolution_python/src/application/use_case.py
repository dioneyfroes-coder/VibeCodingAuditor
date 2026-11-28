import httpx
import PIL.Image
import sys
from domain.entity import Entity

cliente = httpx.Client()
imagem = PIL.Image.new("RGB", (1, 1))
entidade = Entity()
versao = sys.version
