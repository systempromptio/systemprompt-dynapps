# Config global de docker-deployment — editable.
#
# ESTE es el unico sitio donde se declaran rutas del host. La skill no lleva
# ninguna ruta local escrita a fuego: todo lo que apunta a tu maquina sale de
# aqui. Se crea en ~/.config/docker-deployment/defaults.env la primera vez que
# corres new.sh, y en actualizaciones posteriores solo se le ANADEN las claves
# nuevas (nunca se sobrescribe lo que hayas puesto).
#
# Las variables *_TEMPLATE son PLANTILLAS: {MAJOR} se sustituye por la version
# mayor de Odoo (14, 16, 17, 18, 19...) al generar cada proyecto.
#
# - Variable vacia o comentada  -> ese montaje no se anade.
# - Ruta que no existe en el host -> se omite el montaje (warning) y el compose
#   se genera igual, sin fallar.
#
# Cada valor admite sobreescritura por variable de entorno: si exportas
# OCA_SRC=/donde/sea, manda eso sobre el default de la derecha.
#
# Ejemplos de layouts alternativos:
#   PROJECTS_ROOT=$HOME/clientes
#   ODOO_SRC_TEMPLATE=$HOME/odoo/{MAJOR}/addons
#   OCA_SRC_TEMPLATE=/srv/oca/{MAJOR}
#   INDAWS_SRC_TEMPLATE=

# Carpeta raiz donde new.sh crea los proyectos cuando no pasas --path.
PROJECTS_ROOT=${PROJECTS_ROOT:-/opt/PROYECTS}

# Fuente de Odoo CE. Solo se monta en v19+ (en <=18 viene dentro de la imagen).
ODOO_SRC_TEMPLATE=${ODOO_SRC:-/opt/odoo}/odoo{MAJOR}/addons

# Repos de addons que se montan read-only en /addons/*.
EE_SRC_TEMPLATE=${ODOO_ENTERPRISE_SRC:-/opt/odoo_enterprise}/{MAJOR}
OCA_SRC_TEMPLATE=${OCA_SRC:-/opt/OCA}/v{MAJOR}
INDAWS_SRC_TEMPLATE=${INDAWS_SRC:-${PROJECTS_ROOT:-/opt/PROYECTS}/indawsodoo}/{MAJOR}
