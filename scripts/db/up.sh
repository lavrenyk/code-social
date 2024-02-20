# run MySQL in Docker
docker run -d -p 3306:3306 \
 -e MYSQL_ROOT_PASSWORD=kakeepoo \
 -e MYSQL_USER=lavrenyk \
 -e MYSQL_PASSWORD=kakeepoo \
 -e MYSQL_DATABASE=social \
 --name mysql \
 mysql