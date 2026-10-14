services:
  db:
    image: postgres:${PG_VERSION}
    container_name: ${PROJECT_NAME}_db
    environment:
      POSTGRES_USER: ${DB_USER}
      POSTGRES_PASSWORD: ${DB_PASSWORD}
      POSTGRES_DB: postgres
    ports:
      - "${DB_PORT}:5432"
    volumes:
      - pgdata:/var/lib/postgresql/data
    shm_size: 1gb
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ${DB_USER}"]
      interval: 5s
      timeout: 5s
      retries: 10
    restart: unless-stopped

  odoo:
    image: indaws/odoo:${ODOO_VERSION}
    container_name: ${PROJECT_NAME}_odoo
    depends_on:
      db:
        condition: service_healthy
    ports:
      - "${ODOO_PORT}:8069"
      - "${LONGPOLL_PORT}:8072"
      - "${DEBUGPY_PORT}:3000"
    environment:
      ADMIN_PASSWD: ${ADMIN_PASSWD}
      HOST: db
      PORT: 5432
      USER: ${DB_USER}
      PASSWORD: ${DB_PASSWORD}
      I_KNOW_WHAT_IM_DOING: "TRUE"
      # El entrypoint de indaws/odoo calcula --workers=<nproc*2> y fuerza
      # --limit-memory-soft/hard cuando ODOO_STAGE no es development|staging,
      # pisando lo que diga odoo.conf (la CLI gana sobre el fichero).
      ODOO_STAGE: ${ODOO_STAGE}
      # Consumidas por el shim /var/lib/odoo/.local/bin/odoo, que up.sh siembra
      # en el volumen. El shim anade estos flags DESPUES de los del entrypoint,
      # y en Odoo gana el ultimo --workers de la linea de comandos.
      ODOO_WORKERS: ${ODOO_WORKERS}
      ODOO_CRON_THREADS: ${ODOO_CRON_THREADS}
      ODOO_LIMIT_MEM_SOFT: ${ODOO_LIMIT_MEM_SOFT}
      ODOO_LIMIT_MEM_HARD: ${ODOO_LIMIT_MEM_HARD}
      ODOO_SERVER_WIDE_MODULES: ${ODOO_SERVER_WIDE_MODULES}
${CUSTOM_ADDONS_ENV}    volumes:
      - filestore:/var/lib/odoo
${ODOO_SRC_MOUNT_LINE}      - ./config/odoo.conf:/etc/odoo/base/odoo.conf:ro
      - ./addons/custom:/addons/custom
${MOUNT_LINES}    tty: true
    restart: unless-stopped

volumes:
  pgdata:
    name: ${PROJECT_NAME}_pgdata
  filestore:
    name: ${PROJECT_NAME}_filestore

networks:
  default:
    name: ${PROJECT_NAME}_net
    driver: bridge
    driver_opts:
      # NAT de salida. "false" = el contenedor NO puede salir a internet
      # (verificado: los puertos publicados siguen funcionando, a diferencia
      # de `internal: true`, que los rompe). Lo maneja net.sh: abierto en el
      # dia a dia, cerrado durante la cuarentena del restore.
      com.docker.network.bridge.enable_ip_masquerade: "${NET_MASQUERADE}"
    # Subred fija por slot. Sin esto, con muchos proyectos docker agota los
    # address pools por defecto y `up` falla con
    # "all predefined address pools have been fully subnetted".
    ipam:
      config:
        - subnet: ${NET_SUBNET}
