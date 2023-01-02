layout: post  
title: Recuperar un wd mycloud gen 1
date: 2022-08-17   
excerpt: Si tienes un wd my cloud generación 1 (el que tiene el frontal mas redondito) ya sabrás que ha fecha 15 de abril del 2022 la empresa Western Digital dejó de dar soporte a tu disco de red y ha inhabilitado el acceso remoto, actualizaciones de seguridad y asistencia técnica.
content: |-2
  Si tienes un wd my cloud generación 1 (el que tiene el frontal mas redondito) ya sabrás que ha fecha 15 de abril del 2022 la empresa Western Digital dejó de dar soporte a tu disco de red y ha inhabilitado el acceso remoto, actualizaciones de seguridad y asistencia técnica.
  
  ![https://i.imgur.com/JJ4AaO5.jpg]
  
  En mi caso tras un apagado en caliente perdí la información que tenia y terminé formateando el disco duro con idea de meterlo en otro nas mas formal, pero tras recapacitar un poco decidí dejar este disco en red como segunda copia de seguridad del nas principal. Con lo cual me encontré sin el software de Western Digital en el disco que hace que un disco duro normal y corriente se convierta en un disco duro en red.
  
  Encontré una pagina en la que se puede descargar una iso con la imagen del sistema de Western Digital 
  [https://mega.nz/b9fc16c2-f485-47cb-9ad7-d61e2faa5d14](https://mega.nz/b9fc16c2-f485-47cb-9ad7-d61e2faa5d14)
  
  El siguiente paso es grabar la imagen descargada mycloud3T.7z en tu disco duro, en mi caso tiene 4 TB de capacidad y la imagen es de un dispositivo de 3 TB pero no hay problema, en todo este proceso he descubierto que se puede ampliar el disco duro a la capacidad que queramos. Para quemar la imagen he usado una distribución linux desde un usb con ventoy que me permite tener en dicho usb las imagenes iso que quiera llevar siempre conmigo. Os dejo un enlace a la pagina de Yo virtulizador que explica mejor que yo como crear este usb que debe ser un imprescindible en vuestra mochila tecnológica.
  
  [https://www.yovirtualizador.com/posts/2021/08/23/](https://www.yovirtualizador.com/posts/2021/08/23/)
  
  Estando en una distribución linux se abre una consola de comandos y elevamos privilegios.
  
  `sudo su`
  
  Queremos saber cual es nuestro disco western digital por lo que listamos los discos
  
  `lsblk`
  
  
  ![https://i.imgur.com/9pkGTHz.png]
  
  
  En mi caso se ve por la capacidad del disco 3.6 T que está en sdc1 por lo que el comando para grabar la iso sería:
  
  `dd if=mycloud.img of=/dev/sdc1 bs=256M status=progress`
  
  Aunque seguramente sepas que significan que son los parámetros pasados nunca está de más explicarlos, así if es el input file (fichero que vamos a quemar) en mi caso estaba en el directorio que contenia la imagen y por eso no hay ruta, en el parámetro of es output file indicamos la ruta de destino en este caso indico la particion 1 del disco c (sdc1)
  bs es para indicarle el tamaño del bloque en 256 mb que venía así en la pagina donde encontré la información.
  status=progress es para que nos muestra la información procesada hasta el momento por el comando.
  
  Una vez termina el proceso podríamos meter nuevamente el disco con su controladora en la caja y funcionar perfectamente con el software pero con una capacidad de 3 Tb y no la de nuestro disco, ya que como he comentado anteriormente la imagen que he localizado es de 3 Tb pero mi disco es de 4 Tb. Para ello seguimos en linux y abrimos un gestor de particiones como puedo ser gparted. 
  sudo gparted
  
  Importante que lo hagais con permisos de administración que sino el proceso no se puede realizar y tendréis bastantes fallos.
  
  ![https://i.imgur.com/NiSizMt.png]
  
  
  En el entorno gráfico debeis ir al desplegable que hay arriba a la derecha de la pantalla y allí elegir la unidad de disco adecuada, en mi ejemplo era /dev/sdc (3.64 TiB). Al seleccionar vamos a ver todas las particiones que ha creado en el proceso anterior pero la que nos interesa es /dev/sdc4 que vereis que no llega al final del disco teniendo a su derecha espacio sin asignar. Pues bien es sencillo darle todo el disco disponible, boton derecho sobre sdc4 y redimensionar. LLevar la flecha de tamaño a la derecha del todo y asignarle todo el disco disponible. Aplicar y esperar a que termine, no tarda mucho. También se puede hacer mediante consola con el siguiente comando lanzado como usuario administrador o su
  
  
  `e2fsck -f -y -v -C 0 /dev/sdc4`
  
  
  En este punto tenemos el disco sdc4 con mayor tamaño del que estaba formateado por lo que yo formatee desde aqui el volumen sdc4 a ext4.
  
  Una vez termina se puede volver a montar el dico duro en su placa propietaria para que podamos arrancarlo y conectarle el cable de red. Cuando el led esté fijo en azul quiere decir que está listo para funcionar por lo que podemos entrar en su pagina de gestión (seria poner la ip que le ha asignado nuestra red) y podemos ver que tenemos un error o que directamente no nos reconoce la capacidad de nuestro disco a pesar de haber ampliado el volumen. Nos da igual puesto que tenemos que formatear nuevamente desde aqui el dispositivo. Para ello nos vamos a configuracion, luego a utilidades, restaurar valores de fabrica, seleccionamos rápida que la completa puede llegar a tardar demasiadas horas.
  
  ![https://i.imgur.com/WQOFnOu.png]
  
  Con ésto conseguimos tener el disco de red con la capacidad del disco duro que le hayamos metido pero con la versión del firmware antigua concretamente la v03.03.01-156, si queremos actualizarla la primera vez no me dejó hacerlo por la red y tuve que bajar el firmware a local del siguiente enlace y decirle que actulizaba desde fichero.
  
  [https://downloads.wdc.com/nas/sq-040500-342-20190805.deb](https://downloads.wdc.com/nas/sq-040500-342-20190805.deb) 
  
  Que nos lleva a una version v04.05.00 y ya en ésta si pude actualizar desde el dispositivo conectandose a sus servidores, aunque desconozco por cuanto tiempo será así.
  
  
  ![https://i.imgur.com/szNQF03.png]
  
  
  En total no es un proceso largo pero claro, siempre vas con miedo a fastidiar algo pero en este caso no había datos que perder y vas mas relajado en el proceso. Espero que os sea útil y que si lo estais leyendo no hayais perdido datos.
  
  Un saludo, Papá Friki.
    
  Documentación consultada:
  
  Debrick a completely dead MyCloud
  [https://community.wd.com/t/guide-debrick-a-completely-dead-mycloud/92253](https://community.wd.com/t/guide-debrick-a-completely-dead-mycloud/92253)
  
  Upgrade HDD of WD My Cloud 1st gen
  [https://community.wd.com/t/guide-upgrade-hdd-of-wd-my-cloud-1st-gen/255081](https://community.wd.com/t/guide-upgrade-hdd-of-wd-my-cloud-1st-gen/255081)
  
  Como cambiar disco duro a un wdmycloud 1gen
  [https://community.wd.com/t/como-cambiar-disco-duro-a-un-wdmycloud-1gen/252404](https://community.wd.com/t/como-cambiar-disco-duro-a-un-wdmycloud-1gen/252404)
  
  Y darle las gracias a @atareao y @YoVirtualizador por contestarme las numerosas cuestiones que les fui planteando durante el proceso.
