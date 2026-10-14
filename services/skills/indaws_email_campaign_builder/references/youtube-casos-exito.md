# Casos de Éxito Indaws — YouTube

## Playlist Principal
- **URL:** https://www.youtube.com/playlist?list=PL9KT1AOP2NPTdVMjx813ZQGTVo8XiSWu5
- **Nombre:** Casos de éxito INDAWS

## Videos de la Playlist (más recientes primero)

| # | Empresa | Video ID | URL |
|---|---------|----------|-----|
| 1 | Espacio Orgánico | g73Rltg6pFY | https://www.youtube.com/watch?v=g73Rltg6pFY |
| 2 | Prima Protección | G4JSTQTZBhk | https://www.youtube.com/watch?v=G4JSTQTZBhk |
| 3 | Todotrofeo | rxxIy4MiHVE | https://www.youtube.com/watch?v=rxxIy4MiHVE |
| - | Kilnher | FG7pXnI59qY | https://www.youtube.com/watch?v=FG7pXnI59qY |
| - | Frooty | WVmC7I4nnu8 | https://www.youtube.com/watch?v=WVmC7I4nnu8 |
| - | Wiselegal | Kmek5oK1SZc | https://www.youtube.com/watch?v=Kmek5oK1SZc |
| - | Recolim | aHrG3X3LiFA | https://www.youtube.com/watch?v=aHrG3X3LiFA |

## YouTube API Key
Guardada en `/root/clawd/.env.odoo`

## Cómo usar en emails

Los emails no pueden reproducir videos. Usar **thumbnail + enlace**:

```html
<a href="https://www.youtube.com/watch?v=VIDEO_ID" target="_blank" style="display:block;text-decoration:none;">
  <img src="https://img.youtube.com/vi/VIDEO_ID/hqdefault.jpg" 
       alt="Ver caso de éxito" 
       style="max-width:100%;border-radius:8px;border:2px solid #6B68FA;">
  <p style="text-align:center;color:#6B68FA;font-weight:bold;margin-top:10px;">
    ▶️ Ver caso de éxito
  </p>
</a>
```

## Thumbnails disponibles

Para cada VIDEO_ID, YouTube genera thumbnails:
- `https://img.youtube.com/vi/VIDEO_ID/default.jpg` (120x90)
- `https://img.youtube.com/vi/VIDEO_ID/mqdefault.jpg` (320x180)
- `https://img.youtube.com/vi/VIDEO_ID/hqdefault.jpg` (480x360) ← **Recomendado para emails**
- `https://img.youtube.com/vi/VIDEO_ID/maxresdefault.jpg` (1280x720)

## Actualización

Para obtener el último video de la playlist, consultar la playlist en YouTube o usar la API de YouTube.
