# Blog & Forum

Verticales de contenido social. Rutas y líneas verificadas contra
`$ODOO_SRC/odoo18/addons/`: **los números de línea son de v18** y en v17/v19 han derivado
(p.ej. `blog.post._inherit` está en `:155` v17, `:169` v18, `:164` v19). Los nombres de modelo,
campo y ruta sí son estables — **ancla por símbolo, no por línea**.

**Y v19 no difiere solo en `json`→`jsonrpc`**: añade el mixin
`website.page_visibility_options.mixin` (`website/models/mixins.py:162`, `WebsiteMultiMixin.website_id`, campos `header_visible`
y `footer_visible`) a los modelos publicables. **0 ocurrencias en v17 y v18.** Lo heredan
`blog.post` (`website_blog/models/website_blog.py:166`, `BlogPost`) y `event.event` (`website_event/models/event_event.py:27`, `class Event`).
O sea: en v19 un post o un evento pueden ocultar cabecera y pie **por registro**, y eso no existe
antes.

## website_blog

### Modelos clave (`models/website_blog.py`)

| Modelo | _name | _inherit | línea |
|---|---|---|---|
| Blog | `blog.blog` | `mail.thread`, `website.seo.metadata`, `website.multi.mixin`, `website.cover_properties.mixin`, `website.searchable.mixin` | 15-23 |
| Post | `blog.post` | `mail.thread`, `website.seo.metadata`, `website.published.multi.mixin`, `website.cover_properties.mixin`, `website.searchable.mixin` — **+ `website.page_visibility_options.mixin` en v19** | 167-170 (v18); `:155` v17, `:164` v19 |
| Tag | `blog.tag` | `website.seo.metadata` | 138-140 |

- `blog.post` toma su estado de publicacion de `website.published.multi.mixin`
  (campo `website_published`, multi-website). No declares un booleano propio.
- `website_message_ids` se filtra a `message_type == 'comment'` (`website_blog/models/website_blog.py:197`, `class BlogPost`):
  los comentarios web del blog son `mail.message` de tipo comment, no un modelo aparte.

### Controller (`controllers/main.py`, clase `WebsiteBlog`, línea 18)

| Ruta | método | línea |
|---|---|---|
| `/blog` | `blog()` | 166 / 176 |
| `/blog/page/<int:page>` | `blog()` | 168 |
| `/blog/tag/<string:tag>` | `blog()` | 169 |
| `/blog/<model("blog.blog"):blog>` | `blog()` | 171 |
| `/blog/<model("blog.blog"):blog>/<model("blog.post"):blog_post>` | `blog_post()` | 225-293 |
| `/blog/feed` (RSS) | `blog_feed()` | 206-213 |

- `blog_post()` incrementa visitas con `sql.increment_fields_skiplock` (línea 288) —
  no uses `write` para contadores de visita (evita locks).

### Cómo extender el blog

- **Anadir campos a `blog.post`**: módulo con `_inherit = 'blog.post'`. Si el campo
  debe verse en frontend, herencia QWeb de la plantilla del post (delega HTML a
  `odoo-frontend`). El módulo declara `depends: ['website_blog']`.
- **Filtrar/ordenar el listado**: hereda `WebsiteBlog.blog()` y ajusta el `domain`
  o el qcontext antes de `super()`.
- **Comentarios**: se gestionan vía chatter (`mail.thread`); para extenderlos usa
  `odoo-mail-integration`.

## website_forum

Foro con sistema de **karma**: casi toda accion esta gated por umbral de karma.

### Modelos clave

**`forum.forum`** (`website_forum/models/forum_forum.py:17`, `class Forum`, _inherit `mail.thread`,
`image.mixin`, `website.seo.metadata`, `website.multi.mixin`, `website.searchable.mixin`).
Concentra 37 campos `karma_*` (umbrales) y `karma_gen_*` (ganancia). Líneas 99-134.
Umbrales de accion (default) más usados:

| Campo | default | gobierna |
|---|---|---|
| `karma_ask` | 3 | crear pregunta (`website_forum/models/forum_forum.py:108`, `class Forum`) |
| `karma_answer` | 3 | responder (109) |
| `karma_edit_own` / `karma_edit_all` | 1 / 300 | editar (110-111) |
| `karma_close_own` / `karma_close_all` | 100 / 500 | cerrar (113-114) |
| `karma_unlink_own` / `karma_unlink_all` | 500 / 1000 | borrar (115-116) |
| `karma_upvote` / `karma_downvote` | 5 / 50 | votar (118-119) |
| `karma_answer_accept_own` / `_all` | 20 / 500 | aceptar respuesta (120-121) |
| `karma_comment_own` / `_all` | 1 / 1 | comentar (122-123) |
| `karma_flag` | 500 | marcar ofensivo (128) |
| `karma_editor` | 30 | editor completo (imagenes/links) (130) |
| `karma_post` | 100 | auto-validar sin moderacion (133) |
| `karma_moderate` | 1000 | moderar (134) |

**`forum.post`** (`website_forum/models/forum_post.py:19`, `class Post`, _inherit `mail.thread`,
`website.seo.metadata`, `website.searchable.mixin`).
- `state`: 'active' / 'pending' / 'close' / 'offensive' / 'flagged' (líneas 35-40).
- Campos computados `can_*` (115-162): `can_ask`, `can_answer`, `can_accept`,
  `can_edit`, `can_close`, `can_unlink`, `can_upvote`, `can_downvote`, `can_comment`,
  `can_flag`, `can_moderate`, `can_use_full_editor`… comparan `user.karma` con el
  umbral del forum. Úsalos en QWeb/lógica en vez de re-implementar la comparacion.
- Métodos gated (lanzan `AccessError`): `create()` (313-340, ask/answer 330-332),
  `write()` (350-407, accept/edit/state), `validate()` (532, moderate),
  `_flag()` (558), `convert_comment_to_answer()` (657), `message_post()` (761-780,
  comprueba `can_comment` línea 776).

**`forum.post.vote`** (`website_forum/models/forum_post_vote.py:9`, `Vote._name`): voto = registro propio.
`create()` llama `_check_karma_rights()` (94-99, AccessError si falta karma_upvote/
downvote) y `_vote_update_karma()` (101-116) aplica karma al **autor del post**.

**`res.users`** (gamification, `gamification/models/res_users.py:11`, `Users.karma`): campo `karma`
computado y almacenado desde `gamification.karma.tracking`. Para sumar karma usa
`user._add_karma(value, source, reason)` / `_add_karma_batch()` (99-124) — NO
escribas `karma` directamente (rompe el tracking).

### Controller (`controllers/website_forum.py`, clase `WebsiteForum`, línea 23)

| Ruta | método | línea |
|---|---|---|
| `/forum/<forum>/new` | `post_create()` (nueva pregunta) | 412-437 |
| `/forum/<forum>/<post_parent>/reply` | `post_create()` (respuesta) | 412-437 |
| `/forum/<forum>/post/<post>/comment` | `post_comment()` | 439-451 |
| `/forum/<forum>/post/<post>/toggle_correct` | `post_toggle_correct()` (json) | 453-463 |
| `/forum/<forum>/post/<post>/upvote` / `/downvote` | json | 512-524 |
| `/forum/<forum>/post/<post>/flag` | `post_flag()` (json) | 622-624 |
| `/forum/<forum>/validation_queue` / `flagged_queue` / `offensive_posts` | colas moderacion | 529-584 |

### Cómo extender el foro

- **Cambiar umbrales por defecto**: NO edites el modelo; los `karma_*` son campos del
  registro `forum.forum`, configúralos por data XML o desde la UI.
- **Nuevas acciones con karma**: en tu modelo lee `forum.karma_<x>` y compara con
  `self.env.user.karma`, lanzando `AccessError` como hace `forum_post.py`. Para sumar
  karma usa `_add_karma`.
- **Extender el listado/pregunta**: hereda `WebsiteForum.questions()` / `question()`.

## Gotchas (blog & forum)

1. **Publicacion blog vía mixin**: `blog.post` no tiene booleano propio; el control
   es `website.published.multi.mixin.website_published` (`website_blog/models/website_blog.py:169`, `class BlogPost`). En
   multi-website el post puede estar publicado en un site y no en otro.
2. **Contador de visitas con SQL crudo**: `blog_post()` usa `increment_fields_skiplock`
   (`website_blog/controllers/main.py:288`, `WebsiteBlog.blog_post`); no lo reemplaces por `write` (perderías el skip-lock
   y crearías contención).
3. **No escribir `karma` directo**: el campo `karma` de `res.users` está computado y
   almacenado desde `gamification.karma.tracking` (`gamification/models/res_users.py:11`, `Users.karma`).
   Escribirlo a mano lo descuadra; usa `_add_karma`/`_add_karma_batch` (99-124).
4. **`create()` del post lanza AccessError**: crear pregunta/respuesta exige
   `karma_ask`/`karma_answer` (`website_forum/models/forum_post.py:330-332`, `Post.create`). Si creas posts desde código
   con un usuario sin karma, falla; usa `sudo()` o un usuario con karma suficiente.
5. **Comentar exige karma**: `forum.post.message_post()` comprueba `can_comment`
   (`website_forum/models/forum_post.py:776`, `Post.message_post`) — un comentario de chatter normal sobre un post falla si el
   usuario no tiene `karma_comment_*`.
6. **Votar afecta a terceros**: `forum.post.vote.create()` modifica el karma del
   **autor del post votado** (`website_forum/models/forum_post_vote.py:101-116`, `Vote._vote_update_karma`), no del votante; tenlo en
   cuenta al testear.
