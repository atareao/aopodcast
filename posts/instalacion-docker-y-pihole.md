---
title: Instalación docker y pihole
date: 2020-05-15
version: 1
slug: instalacion-docker-y-pihole
excerpt: Buenas muchachada, hoy me estreno con un manual sobre la instalación de docker en una raspberry pi y luego configurar Pihole para evitar publicidad en la navegación.
---
# Instalación de docker

En un par de comandos se deja listo.

``` curl -sSL https://get.docker.com | sh ```

(Opcional pero recomendable) para agregar el usuario docker al grupo pi, para no tener que ejecutar los comandos docker con sudo delante:

```sudo usermod -aG docker pi ```

Si en vuestro caso no usais el usuario pi sustituirlo al final del comando anterior.

## Instalacion de Portainer para controlar los docker

Nuevamente un par de comandos. Con el primero se crea un volumen y se deja preparado para el siguiente comando.

```sudo docker volume create portainer_data```


Y con éste otro lo que se hace es bajar la imagen si no estaba en el equipo (como es la primera vez no va a estar) y se crea el contenedor:


``` sudo docker run -d -p 9000:9000 -v /var/run/docker.sock:/var/run/docker.sock -v portainer_data:/data portainer/portainer ```

Ya podemos acceder a la ip de la raspberry y al servicio del puerto 9000 que es el que le hemos indicado anteriormente. Algo así si vuestra raspberry esta en la ip 120

```http://192.168.1.120:9000/#/auth```


## Instalacion de pihole en docker

Rápido con un solo comando. Llevará un tiempo ya que la imagen del contenedor no la tenemos en local y tiene que descargarla. Éste paso seria posible hacerlo desde la consola grafica de Portainer pero sinceramente he aprendido a hacerlo por consola y es muy sencillo.

```
 docker run -d \
    --name pihole \
    -p 53:53/tcp \
    -p 53:53/udp \
    -p 67:67/udp \
    -p 80:80 \
    -p 443:443 \
    -e PUID=0 -e PGID=0 \
    -e TZ=Europe/madrid \
    -v /mnt/pihole/pihole/:/etc/pihole/ \
    -v /mnt/pihole/dnsmasq.d/:/etc/dnsmasq.d/ \
    -e 127.0.0.1 \
    -e 1.1.1.1 \
    --restart=unless-stopped \
    pihole/pihole:latest
```


Paso opcional pero recomendable cambiar la contraseña de pihole. Hay que correr el docker con shell para ello:

``` docker exec -i -t pihole /bin/bash ```

Dentro de la shell del docker cambiar la contraseña con el comando de abajo, os pide la contraseña y confirmarla:

```pihole -a -p ```

Ha sido fácil, ya solo falta salir de ésta shell para ello:

```exit```


## Configuración de pihole

Lo primero es ir ya en el navegador web a la ip de tu raspbery y entrar con la contraseña del paso anterior.

```http://TU_IP_RASPBERRY/admin```

Ahora hay que añadir las listas que harán que la publicidad desaparezca casi por completo. Para ello hay que ir a al panel de la izquierda a la sección Group Management y Addlist o bien entrar por la dirección que os dejo abajo

```http://TU_IP_RASPBERRY/admin/groups-adlists.php```


Y las listas que tengo metidas son las siguientes, has de ir añadiendo de una en una. La última es la que le comentaba a Converso72 en SumandoPodcast que añade muchísimas excepciones:

```https://raw.githubusercontent.com/StevenBlack/hosts/master/hosts
https://mirror1.malwaredomains.com/files/justdomains
http://sysctl.org/cameleon/hosts
https://zeustracker.abuse.ch/blocklist.php?download=domainblocklist
https://s3.amazonaws.com/lists.disconnect.me/simple_tracking.txt
https://s3.amazonaws.com/lists.disconnect.me/simple_ad.txt
https://hosts-file.net/ad_servers.txt
https://dbl.oisd.nl/
```

Espero que os haya sido facil la instalación y lo disfrutéis. En mi caso la raspberry es la que resuelve los DNS por lo que aparato que quiero que disfrute de navegación sin publicidad le pongo ip fija y le digo que resuelva sus dns la ip de la raspberry.

Un saludo nos vemos, nos leemos, nos escuchamos.
