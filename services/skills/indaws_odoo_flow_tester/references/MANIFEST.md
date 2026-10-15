# MANIFEST de las referencias repartidas

Cada fichero de `flows/*/` y `revert/*/` es UNA seccion del monolito
que habia antes, con su texto intacto. Concatenar los de un subdirectorio en
orden de nombre reproduce el fichero original carácter a carácter; el md5 de
abajo es de ese original y `tests/test_flows.py` lo comprueba en cada ejecución.
Si alguien borra o trunca un fichero, ese md5 deja de cuadrar.

Una edición legítima del catálogo también lo rompe, y es deliberado: se re-sella
con `scripts/flows.py --sellar`, y la línea de md5 que cambia en el diff es la
señal de revisión de que se ha tocado contenido, en vez de que el cambio se
cuele entre 171 ficheros.

Verificado el día del reparto contra `git show HEAD:<monolito>`: idéntico byte a
byte en los seis.

| original | subdirectorio | md5 | bytes | ficheros |
|---|---|---|---|---|
| `atomic_flows.md` | `flows/atomic/` | `556afb52c1bb7879eb3fbda14f7e84a0` | 136158 | 39 |
| `extended_models.md` | `flows/extended/` | `6ccbb8d8294a33d467e9aa5dd0ec13b6` | 36793 | 25 |
| `flow_chains.md` | `flows/chains/` | `05efdca1f466f3ab81d5214f51817de9` | 134501 | 42 |
| `revert_protocol.md` | `revert/protocol/` | `849cd5408e2c59acc96d78fc915115d7` | 78556 | 21 |
| `sql_cleanup.md` | `revert/sql/` | `0372a411d07bcfbba5598a0d4a185535` | 94157 | 34 |
| `sequence_restoration.md` | `revert/sequences/` | `41c8de7359ad17d73112b22f35186e18` | 23188 | 10 |
