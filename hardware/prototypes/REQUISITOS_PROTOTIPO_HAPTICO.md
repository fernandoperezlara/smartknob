# Especificación de diseño del prototipo háptico

Fecha: 2026-09-10. Revisión documental: 1.0.

## 1. Propósito y estado de este documento

Este documento es el encargo técnico para un agente o diseñador externo que debe desarrollar el esquema y la PCB de una plataforma de evaluación de motores para un mando háptico. Debe poder trabajar sin acceder a la conversación que originó los requisitos.

**Es una especificación, no un esquema validado ni una autorización para fabricar o comprar.** El diseñador debe convertir los requisitos en un diseño calculado y verificable. No debe interpretar valores preliminares como capacidades comprobadas ni declarar ensayos físicos que no haya realizado.

Convenciones:

- **DEBE / NO DEBE:** requisito obligatorio.
- **Propuesta:** punto de partida que el diseñador puede ajustar con cálculos y justificación, sin cambiar la arquitectura acordada.
- **Pendiente de cierre:** decisión que debe quedar resuelta y documentada antes del hito indicado; no ocultarla con un valor inventado.

El encargo incluye diseño y documentación. Cambiar la familia ESP32-C6, el driver, integrar el motor, añadir batería o ampliar el rango de alimentación requiere aprobación del propietario.

## 2. Objetivo funcional

La plataforma debe permitir comparar motores BLDC trifásicos pequeños de gimbal y experimentar con:

1. Dos posiciones estables ON/OFF, retorno hacia posiciones definidas y topes virtuales.
2. Detentes o clics distribuidos angularmente, con giro multivuelta sin límite mecánico.
3. Retorno elástico, fricción y amortiguación configurables.
4. Deshabilitación del accionamiento para evaluar fricción y cogging del motor sin excitación.

Los topes son fuerzas virtuales limitadas: no son bloqueos mecánicos, y el usuario puede vencerlos. La sensación depende del motor, su montaje, el sensor y el control; una PCB correcta no garantiza por sí sola un tacto determinado.

FOC es un algoritmo ejecutado por el microcontrolador. No existe aquí un «motor FOC» con control autónomo. El driver conmuta las tres fases siguiendo PWM.

## 3. Alcance y exclusiones

### 3.1 Requisitos acordados

| ID | Requisito |
|---|---|
| ARQ-01 | PCB portadora con esquema, para uso de laboratorio y un motor cada vez. |
| ARQ-02 | Placa de desarrollo ESP32-C6 extraíble mediante zócalos hembra; sin soldar el módulo a la portadora. |
| ARQ-03 | Driver exacto Texas Instruments **DRV8316CRRGFR**, variante SPI. |
| ARQ-04 | Control por **3 PWM** y SPI de configuración/diagnóstico. |
| ARQ-05 | Motor externo, conectado a U/V/W; sin motor ni agujeros de fijación del motor en la portadora. |
| ARQ-06 | Encoder **MT6701 externo**, conectado mediante conector desmontable. |
| ARQ-07 | SPI compartido entre driver y encoder, con selecciones independientes. |
| ALI-01 | Entrada externa de potencia **5–12 V CC**, ajustable en una fuente de laboratorio. |
| ALI-02 | ESP32-C6 alimentado desde su propio USB-C; no desde la entrada del motor. |
| ALI-03 | Masa común; positivos USB y potencia separados. |
| ALI-04 | Sin convertidor buck/boost ajustable para el motor en esta revisión. |
| SEG-01 | Entrada protegida, control de transitorios y previsión completa de descarga regenerativa. |
| SEG-02 | Deshabilitación física y estado seguro durante reset, arranque y alimentación parcial. |
| MED-01 | Prever adquisición de corrientes para FOC en corriente, además de lectura de VM y del encoder. |

La antigua idea de usar el XIAO no obliga a emplearlo: la decisión más reciente es una placa ESP32-C6 desmontable con suficientes pines. **No se permite sustituir un footprint por otro «ESP32-C6 genérico».**

### 3.2 Fuera de alcance

- Pantalla, batería, cargador, USB Power Delivery, alimentación autónoma y carcasa final.
- Alimentar el ESP32-C6 con el regulador interno del driver.
- Compatibilidad universal con cualquier motor, incluidos motores de escobillas, steppers o motores con ESC integrado.
- MKS 5010 de baja resistencia como carga garantizada de esta revisión.
- Diseño de fijación del motor o PCB del propio encoder. Sí se documentará su interfaz y montaje necesario.

Los agujeros para separadores de la **portadora** sí pueden incluirse; no son fijaciones de motor.

## 4. Contexto del repositorio y preservación

Ubicaciones relativas a la raíz:

- `hardware/prototypes/driver/`: proyecto KiCad existente. **No ha sido auditado para esta especificación.** No asumir que corresponde al driver exacto, pinout o alimentación de este documento.
- `hardware/smartknob/`: diseño distinto; no modificar por defecto.
- `hardware/libraries/`: bibliotecas compartidas; preservar cambios existentes.
- `firmware/`: firmware Rust para ESP32-C6, no un proyecto Arduino/SimpleFOC ya operativo.
- `README.md`: describe lectura del MT6701 y pantalla, y advierte que el control del motor no está implementado.
- `docs/datasheets/`: documentación local; comprobar modelo y revisión. El nombre `DRV8316.pdf` no garantiza que documente el **DRV8316C** requerido.

Antes de trabajar: leer instrucciones AGENTS aplicables, revisar estado de Git, exportar/revisar el diseño previo y decidir qué se reutiliza. No sobrescribir trabajos ajenos, eliminar copias ni forzar cierres de KiCad. En la inspección documental había archivos de bloqueo en el proyecto `driver`; comprobar de nuevo su estado, no borrarlos automáticamente.

Si se usa Konnect, seguir sus reglas de edición de KiCad; no manipular directamente sus archivos serializados como texto. Mantener bibliotecas portables dentro del repositorio. La información de este documento prevalece sobre suposiciones inferidas de un diseño antiguo, pero no autoriza destruirlo.

## 5. Arquitectura eléctrica

```text
Fuente de laboratorio 5–12 V
           |
     J_PWR, fusible, protección de polaridad
           |
       VM protegida ----- bulk / TVS / descarga regenerativa
           |
       DRV8316CR -------- J_MOTOR U/V/W -------- motor BLDC externo
           ^                                      |
       3 PWM + SPI                            acoplamiento mecánico
           |                                      |
USB-C -> placa ESP32-C6 ---- J_ENC SSI/SPI ---- MT6701 + imán externo
           |
      ADC de corrientes y VM; control seguro; diagnóstico

GND común, con retornos de potencia y señal diseñados deliberadamente.
```

VM es la alimentación de potencia del driver, no una segunda tensión independiente para el motor. Las fases son salidas conmutadas y no se conectan al positivo de la fuente directamente.

## 6. Decisiones que el diseñador debe cerrar

| Decisión | Criterio / propuesta | Hito límite |
|---|---|---|
| Placa ESP32-C6 exacta | Evaluar como candidata la **Espressif ESP32-C6-DevKitC-1** oficial; confirmar revisión, esquema, dimensiones, USB-C, headers y disponibilidad. No es una compra aprobada. | Antes de elegir zócalos y footprint. |
| Corrientes de diseño | Propuesta de cálculo: **1 A RMS continuo por fase** y **2 A instantáneos por fase durante hasta 100 ms**, con intervalo entre pulsos definido por evaluación térmica. Son objetivos de la portadora, no límites seguros de cada motor ni prestaciones verificadas. | Antes de cerrar potencia, filtros, fusible y PCB. |
| Corriente de entrada | Calcular separadamente de la corriente de fase; definir valor continuo, pico e inrush. | Antes de seleccionar J_PWR y fusible. |
| Temperatura | Propuesta: evaluar a 25 °C ambiente sin ventilación forzada; fijar límites y márgenes para chip, conectores, MOSFET y resistencias. | Antes del informe térmico. |
| Firmware de control | Priorizar compatibilidad con Rust existente; demostrar soporte real PWM/ADC. No asumir que citar SimpleFOC resuelve el control en Rust. | Antes de congelar pines y fabricar. |
| Corriente medida | Preferir tres canales; dos solo con reconstrucción y ventanas de muestreo demostradas. | Antes del esquema definitivo. |
| Dimensiones y capas | Propuesta: 4 capas, FR-4 de 1,6 mm, prioridad a acceso y disipación sobre miniaturización. Dimensionar al módulo seleccionado. | Antes del layout. |
| Regeneración | Fijar tensión de activación, histéresis, energía/potencia admisible y componentes del circuito de descarga. | Antes de congelar protecciones. |

El diseñador puede resolver elecciones ordinarias de componentes y geometría con justificación. Si los objetivos anteriores son inviables sin cambiar alcance, debe solicitar decisión. No presentar una placa a fabricar con corriente, módulo o protección aún «por definir».

## 7. Microcontrolador, pines y contrato de firmware

El ESP32-C6 dispone de un SPI de propósito general; una placa más grande expone más GPIO pero no añade otro controlador SPI. Consultar los documentos [E1–E3].

Crear una tabla definitiva con **señal, pin físico del zócalo, GPIO, función/periférico, estado de arranque, dirección, nivel eléctrico y restricciones**. Verificar strapping, USB, memoria, LED y circuitos incluidos en la placa de desarrollo.

Presupuesto orientativo, no pinout definitivo:

| Señal lógica | Recursos |
|---|---|
| PWM_U, PWM_V, PWM_W | 3 salidas PWM sincronizadas. |
| SPI_SCLK, SPI_MOSI, SPI_MISO | 3 GPIO del SPI compartido. |
| CS_DRV_N, CS_ENC_N | 2 salidas independientes. |
| I_U, I_V, I_W | Preferentemente 3 entradas ADC. |
| VM_SENSE | 1 entrada ADC protegida y escalada. |
| MOTOR_ARM, DRV_SLEEP_N | Prever 2 salidas; cualquier combinación debe justificarse. |
| DRV_FAULT_N | 1 entrada de diagnóstico. |
| Temperatura / estado del interruptor / heartbeat | Reserva según solución de seguridad. |

No asignar pines por intuición ni por equivalencia con ESP32 clásico/S3. No reutilizar pines USB. Un expansor GPIO lento no sustituye entradas de corriente, SPI del encoder ni PWM.

La adquisición para FOC debe probarse con una implementación mínima: generación PWM, instante de conversión, lectura de encoder, tiempos de ejecución y respuesta a fallo. Si el ADC integrado/SDK elegido no permite cumplirlo, proponer adquisición externa o cambio aprobado de plataforma; no prometer FOC en corriente a partir de `analogRead` asíncrono sin análisis.

## 8. Driver DRV8316CRRGFR

Consultar [T1] para el sufijo exacto. Hechos críticos: SPI configura/diagnostica; 3 PWM acciona; SOA/SOB/SOC son medidas analógicas de corriente en los transistores inferiores. El modo 3 PWM normal permite SOx; el modo con limitación ciclo a ciclo no. DRVOFF alto deshabilita las salidas. El rango operativo del chip no equivale al de esta PCB.

Requisitos de implementación propios del proyecto:

- Preferir 3 PWM normal con lectura de corriente. Documentar la elección de VREF y los límites de ganancia/ADC. **No exigir simultáneamente SOx disponibles y limitación ciclo a ciclo interna.**
- Verificar y documentar las conexiones de INHx e INLx y su estado antes de configurar registros. No dejarlas flotantes.
- Justificar referencia, filtrado, calibración de offset, polaridad, adquisición y saturación de cada canal analógico.
- Mantener OCP/diagnóstico; no confundir protección ante fallos de potencia con un límite preciso y bajo que proteja cualquier bobinado.
- Documentar secuencia de configuración, lectura de comprobación y habilitación. No cambiar modos mientras se acciona el motor.
- Verificar por tabla completa pin físico–símbolo–pad todos los pines, pads repetidos, masas y pad térmico del encapsulado exacto.
- Revisar nFAULT en arranque con USB ausente: [T1] advierte una condición de entrada a modo de prueba si su pull-up no cumple el arranque especificado. No depender ciegamente del 3,3 V del micro.
- No unir AVDD a la salida 3,3 V del módulo. Resolver las señales entre dominios sin retroalimentación.
- Determinar el tratamiento documentado de los pines del buck interno cuando no se utiliza; no omitir componentes ni conectar pines por analogía con otra variante.

Aplicar valores y conexiones de auxiliares desde la tabla de componentes externos de [T1], comprobando capacidad efectiva, tolerancias y tensión diferencial real. No sustituir esa revisión por un «100 nF en todos los pines».

## 9. Interfaz del MT6701 y motor

El MT6701 debe utilizar SSI leído con el SPI del micro. La trama incluye ángulo, estado y CRC [M1]. Usar el firmware existente como referencia de decodificación, no como prueba de tiempo real del nuevo control.

- Lógica y alimentación del módulo de encoder: 3,3 V, verificando la placa externa real y sus pull-ups/regulador.
- Conexiones funcionales mínimas: GND, 3V3_ENC, SCLK, CS_ENC_N y MISO/DO. No requiere MOSI.
- Documentar cómo se selecciona SSI en el módulo externo y qué pinout tiene; un breakout que solo expone I²C no sirve sin modificaciones.
- Compartir reloj y MISO con el driver; garantizar alta impedancia cuando no esté seleccionado y ausencia de contención durante arranques parciales. Incorporar buffer con aislamiento de alimentación si la evaluación lo exige.
- Cada CS debe quedar inactivo sin firmware. No activar ambos a la vez.
- Punto de partida de encoder: modo SPI 1, 1 MHz, 24 bits completos, según README; verificar frente a [M1] y capturas.
- Diseñar para cables cortos de banco; objetivo inicial ≤20 cm para encoder. Documentar longitud realmente validada. Prever resistencias serie/ESD de baja capacidad donde lo justifique la señal.
- Preferir un conector polarizado de 6 posiciones con GND adicional si facilita retorno. El pinout físico lo debe fijar el diseñador, imprimirlo y comprobar orientación de acoplamiento; no es compatible automáticamente con otros módulos.
- J_MOTOR debe tener 3 contactos de potencia identificados U/V/W, con retención. Seleccionar corriente, cable y adaptadores para el sobre de diseño.

No conectar o desconectar motor ni encoder con la potencia activa. El motor debe estar sujeto en un soporte externo estable. El imán debe estar centrado, con magnetización, distancia y campo compatibles; no asumir que cualquier anillo magnético sirve para el MT6701.

La compatibilidad de motores se evaluará individualmente con resistencia (indicando fase o entre terminales), inductancia si está disponible, pares de polos, corriente, mecánica y temperatura. No inferir datos de otro producto llamado «2804».

## 10. Alimentación y protección

### 10.1 Entrada y dominios

La entrada nominal admitida es 5–12 V CC. El mínimo debe cumplirse en VM bajo carga, descontando cables y protección; verificar margen de subtensión. No etiquetar el conector como 35 V. No usar 40 V absolutos del chip como tensión de trabajo.

Definir y calcular en un cuadro: rango normal, tolerancia de fuente, máxima tensión transitoria de VM, umbrales de descarga, clamping del TVS y rating de cada componente. El componente más débil limita el sistema.

Requeridos: fusible reemplazable o solución equivalente justificada, MOSFET contra inversión con protección de compuerta cuando corresponda y conector inequívoco. Revisar orientación del diodo de cuerpo, corriente inversa, SOA, disipación e inrush. **Un MOSFET contra polaridad inversa no garantiza bloqueo de corriente hacia la fuente.**

Decidir explícitamente si se bloquea retorno a la fuente. Si se bloquea, bulk y descarga regenerativa deben quedar del lado del driver. No bloquear la única vía de absorción y declarar que la sobretensión desaparece.

ESP32 alimentado solo desde su USB-C. Derivar alimentación del encoder desde lógica si la capacidad está verificada. Si se necesita una alimentación auxiliar de seguridad desde VM, será de baja potencia y no alimentará el ESP32; documentar su dominio.

### 10.2 Condensadores y transitorios

- Cerámicos y auxiliares según circuito recomendado del componente exacto.
- Bulk situado cerca del bucle de potencia. Como exploración, reservar espacio para decenas a centenas de µF; el valor instalado se decide por cálculo/medición, no por esta frase.
- Considerar ESR, corriente de rizado, temperatura, polaridad, inrush y pérdida de capacidad por sesgo DC.
- Propuesta de familias de tensión: evaluar al menos 25 V para condensadores entre VM y GND, elevando a 35 V si el clamping/margen lo exige. Esto no impone el mismo rating a condensadores entre nodos de bomba de carga.
- El TVS debe tolerar la máxima tensión normal sin conducir y limitar por debajo del sobre transitorio calculado, considerando su corriente de pulso y tolerancias. No seleccionar por la etiqueta «TVS de 12 V».
- Coordinar fusible y TVS ante fallo permanente. Un TVS no protege indefinidamente frente a una fuente ajustada incorrectamente.

### 10.3 Energía regenerativa

La fuente de banco puede entregar corriente y no absorberla. Analizar frenado, movimiento manual con driver deshabilitado y fuente desconectada. DRVOFF no elimina las rutas pasivas del puente.

Debe incluirse el **diseño completo y footprint** de descarga: detector/comparador con referencia, histéresis, MOSFET, resistencia de potencia, mando y alimentación. Preferir activación autónoma desde el dominio VM, operativa sin USB. Puede proponerse resistencia externa si disipación y acceso lo requieren.

El montaje DNP de esa función solo será admisible si se justifica la energía máxima tolerada por bulk/TVS y se restringen/documentan los ensayos. No omitir silenciosamente el circuito. No depender de software para la única protección de sobretensión.

Calcular al menos:

```text
E_absorbida_por_C = 0,5 * C * (V_limite² - V_inicial²)
I_descarga = V_bus / R_descarga
P_descarga = V_bus² / R_descarga
```

Fijar un sobre de energía/potencia de ensayo; no se conoce aún la inercia ni cuánto trabajo puede aportar el usuario. Incluir tolerancias, pulsos y capacidad térmica de resistencia/MOSFET. Un umbral fijo por encima de 12 V protege un techo de VM; **no mantiene una prueba de 5 V exactamente en 5 V**. Medir VM y explicar esa diferencia al usuario.

### 10.4 Apagado y alimentación parcial

Implementar un circuito de habilitación que requiera permiso del micro y posición habilitada del interruptor; cualquier condición de fallo debe dominar. No conectar un interruptor a una salida push-pull de modo que cortocircuite el GPIO. Añadir pull-ups/pull-downs y aislamiento de dominios según cálculo.

| Situación | Comportamiento requerido |
|---|---|
| USB y VM ausentes | Sin accionamiento; considerar descarga de bulk residual. |
| Solo USB | Lógica operativa; sin alimentación fantasma del driver por SPI/PWM. |
| Solo VM | Driver deshabilitado; sin alimentar el ESP32 a través de GPIO; protección regenerativa funcional. |
| USB primero, después VM | Sin movimiento antes de configuración, comprobación y armado explícito. |
| VM primero, después USB | Igual; verificar especialmente señales de arranque y nFAULT. |
| Reset, brownout o desconexión USB | Retirada segura de habilitación; no conservar PWM activo. |
| Interruptor OFF | Deshabilitación independiente del firmware. |
| Fuente retirada, motor en movimiento | Sin sobretensión fuera del sobre calculado. |

Un GPIO que conserva nivel alto no detecta por sí solo firmware colgado. Prever watchdog y demostrar retirada de habilitación; si no se logra un tiempo acotado, incorporar monitor de heartbeat independiente. El apagado físico no es una función de seguridad industrial certificada.

## 11. Control, adquisición y comportamiento ante fallos

La portadora debe permitir evolucionar de pruebas limitadas por tensión a FOC en corriente. En ningún caso etiquetar estimaciones V/R como corriente medida.

- Elegir frecuencia PWM y estrategia de modulación con ventanas válidas de muestreo. Propuesta inicial: 20–25 kHz de PWM, ajustable tras verificar ruido, pérdidas y tiempos.
- Programar ADC dentro de ventanas válidas, lejos de conmutaciones; justificar tiempos de establecimiento y desfase entre canales. Las medidas de low-side no están disponibles indistintamente en todo el ciclo.
- Fijar frecuencia del lazo de corriente y del lazo háptico mediante presupuesto de tiempos; no prometer 20 kHz de control por generar PWM a 20 kHz. Objetivo inicial de evaluación háptica: 1 kHz, revisable por ensayo.
- Mantener SPI del encoder prioritario; diagnóstico y logs fuera de las secciones críticas. No hay pantalla en esta revisión.
- Medir VM para ajustar normalización y detectar caídas/sobretensión. Proteger la entrada ADC en todos los estados de alimentación.
- Perfil por motor: pares de polos, límite de corriente, límite de tensión, offset/dirección del sensor, temperatura y ganancias.
- Primera calibración con tensión/fuerza reducida. No ejecutar búsqueda de posición a potencia máxima.
- Validar CRC, estado magnético, continuidad angular y tiempo desde la última lectura. Nunca continuar indefinidamente con el último ángulo válido.
- Propuesta de seguridad: retirar el permiso de accionamiento ante encoder inválido o vencimiento de un plazo de 2 ms; medir latencia y justificar otro plazo si fuera necesario. Un ruido intermitente debe diagnosticarse, no silenciarse debilitando la protección.
- Fallos enclavados con rearmado explícito; no arrancar repetitivamente tras OCP o pérdida del encoder.
- Registrar VM, corrientes, ángulo, estado y causa de deshabilitación sin bloquear el control.

Prever conexión opcional de termistor externo y/o método de medida térmica de banco. La protección térmica del driver no sustituye la vigilancia del motor. La fuente limitada tampoco limita directamente cada corriente de fase.

## 12. PCB, mecánica y puntos de prueba

Propuesta de organización: módulo/antena y encoder en zona de señal; driver, entrada, bulk y motor agrupados en zona de potencia; descarga térmicamente separada de conectores, dedos y encoder.

- Respetar keep-out de antena del módulo y acceso a USB, BOOT y RESET.
- Comprobar distancia entre filas de zócalos, altura, orientación y extracción. Dibujar contorno del módulo en capa mecánica y documentar montaje.
- No rutear señales sensibles debajo de nodos de conmutación. Mantener referencias de masa continuas; no abrir cortes arbitrarios de plano bajo señales.
- Minimizar bucle bulk–puente–retorno y rutas de bomba de carga; seguir layout del fabricante.
- Dimensionar cobre, vías y conectores por temperatura/corriente calculadas. No fijar «pistas de 1 mm» como regla universal.
- Resolver pad térmico con estrategia de vías compatible con ensamblaje; no especificar via-in-pad abierto sin evaluar pérdida de soldadura.
- Preferir pasivos manejables y puntos de prueba accesibles; la soldadura del QFN exige proceso adecuado, no un simple montaje de headers.
- Serigrafía: revisión, 5–12 V, polaridad, U/V/W, pin 1 del encoder, tensión lógica y posición OFF.
- Test points mínimos: GND junto a señales, VM antes/después de protección, 3V3, SOA/B/C, VM_SENSE, PWM_U/V/W, CS de ambos dispositivos, SCLK, MISO, MOSI, DRVOFF, nSLEEP y nFAULT.
- No unir la pinza de masa de un osciloscopio con tierra a una fase; las medidas entre fases requieren instrumentación diferencial adecuada.

## 13. Flujo de trabajo del agente diseñador

1. **Inventario:** inspeccionar repositorio/proyecto previo sin modificarlo; registrar reutilización y discrepancias.
2. **Cierre de arquitectura:** elegir placa exacta, sobre de corriente, adquisición, protección regenerativa y dimensiones propuestas. Resolver incompatibilidades, no ocultarlas.
3. **Prueba de viabilidad de firmware:** confirmar GPIO, PWM sincronizado, ADC y SPI en el software objetivo. Si no hay hardware, distinguir compilación de validación temporal pendiente.
4. **Cálculos:** entregar presupuesto de pines, tensiones, corrientes, disipación, inrush, energía regenerativa y comportamiento sin USB.
5. **Bibliotecas:** comprobar pin físico–símbolo–pad–vista para cada parte exacta, especialmente driver, MOSFET, zócalos y conectores. Verificar huellas con dibujos dimensionados.
6. **Esquema por bloques:** potencia/protección; driver; módulo; encoder/analógico; seguridad y test points. Documentar DNP, puentes y variantes.
7. **Revisión de esquema:** exportar netlist, ejecutar ERC y revisar visualmente todas las hojas renderizadas. Resolver cortos, conexiones y advertencias con evidencia.
8. **Layout:** colocar primero potencia y mecánica; rutear con reglas calculadas; comprobar térmica, retorno y antena.
9. **Validación:** guardar, rellenar zonas, ejecutar DRC, revisar imágenes/3D y correspondencia con BOM y conectores.
10. **Entrega:** fuentes portables, informes y plan de puesta en marcha. No fabricar/comprar sin autorización explícita.

No confundir un informe automático «sin errores» con un diseño eléctricamente correcto. Ningún ERC/DRC valida por sí solo regeneración, timing ADC, pinout físico o tacto háptico. Las pruebas no ejecutadas deben figurar como pendientes.

## 14. Plan de puesta en marcha y aceptación

| Etapa | Procedimiento | Criterio de aceptación |
|---|---|---|
| Inspección sin tensión | Polaridad, continuidad, cortos, orientación, soldadura QFN y resistencia de descarga. | Sin defectos detectados; resistencias entre alimentación y masa justificadas. |
| Lógica USB | Motor desconectado; verificar 3V3 y estados de habilitación. | Sin alimentación fantasma ni permiso de motor. |
| Potencia sola | Fuente limitada, motor desconectado, barrer 5–12 V de forma controlada. | Lógica de seguridad y arranque correctos, sin calentamiento anómalo. |
| Secuencias | Ensayar todas las filas de la tabla de alimentación parcial. | Sin accionamiento espontáneo ni tensiones indebidas en GPIO. |
| SPI y encoder | Analizador lógico y giro manual. | CS excluyentes; tramas y CRC correctos; bus sin contención. |
| Driver | Configurar deshabilitado y comprobar registros/fallos. | Configuración leída coincide; interruptor domina al firmware. |
| Primer motor | Motor sujeto, corriente/tensión bajas, calibración limitada. | Dirección y offset correctos; corriente controlada. |
| ADC/FOC | Capturar PWM, ventanas ADC y corriente externa de referencia. | Escala, signo, offset y sincronización demostrados. |
| Fallos | Simular ausencia de lecturas en firmware; reset y retirada de USB. No retirar físicamente conectores activos. | Habilitación retirada en plazo documentado; rearmado explícito. |
| Regeneración | Ensayo de energía controlada, fuente que no absorbe y también USB ausente. | VM bajo techo calculado; descarga y temperatura dentro de límites. |
| Térmica | Perfil continuo y pulsado dentro del sobre aprobado; ensayo inicial de 30 min o hasta equilibrio. | Temperaturas con margen documentado; sin derivas o apagados indebidos. |
| Háptica | ON/OFF, retorno, detentes y multivuelta a varias tensiones. | Sin oscilación sostenida; límites respetados; resultados registrados por motor. |

El diseño documental puede entregarse antes de fabricar. Solo se declarará «validado en hardware» tras ejecutar y registrar ensayos. Si falta instrumental o hardware, indicarlo; no sustituir medición por simulación sin identificarla.

## 15. Entregables del diseñador

- Proyecto KiCad editable y portable, esquema PDF y vistas de PCB.
- BOM con MPN exactos, encapsulados, tolerancias, ratings, alternativas verificadas y DNP.
- Tabla de pines del módulo y pinouts de conectores con vista de acoplamiento.
- Informe de cálculo de potencia, protección, térmica, corriente/ADC y regeneración.
- Contrato hardware–firmware: señales, polaridades, registros/modos, secuencias y temporización.
- Resultados ERC/DRC, lista razonada de excepciones y revisión visual.
- Plan de ensamblaje, cableado y primera puesta en marcha; perfiles iniciales conservadores de motor.
- Registro de decisiones, limitaciones y ensayos pendientes.
- Tras aprobación para fabricación: Gerbers, taladros, BOM y posición de componentes consistentes con la misma revisión.

## 16. Lista de cierre antes de fabricación

- [ ] Modelo/revisión real de ESP32-C6 y geometría de zócalos comprobados.
- [ ] Corrientes objetivo, duración de pico e intervalo térmico definidos.
- [ ] Capacidad de entrada y fusible coordinados, sin equiparar corriente de bus y fase.
- [ ] Pinout completo, ADC y software de adquisición viables.
- [ ] Driver exacto y correspondencia física de todos sus pads aceptados.
- [ ] Modo de lectura de corriente compatible con configuración elegida.
- [ ] Arranque de nFAULT, referencia analógica y estados INHx/INLx resueltos.
- [ ] Secuencias de alimentación parcial y ausencia de backfeed analizadas.
- [ ] OFF físico dominante y detección de firmware colgado definida.
- [ ] Desacoplo, TVS, fusible, MOSFET y descarga calculados conjuntamente.
- [ ] DNP de descarga justificado o circuito montado y especificado.
- [ ] Motor/encoder externos, conectores inequívocos y límites documentados.
- [ ] Cobre, térmica, antena, mecánica y ensamblaje revisados.
- [ ] ERC/DRC y revisión visual realizados; pendientes declarados.
- [ ] Aprobación explícita del propietario para fabricar/comprar.

## 17. Fuentes y jerarquía de evidencia

Consultadas el 2026-09-10. Descargar/revisar versiones vigentes al diseñar y registrar las utilizadas. Las fichas oficiales prevalecen sobre anuncios de venta, ejemplos de terceros y heurísticas de revisión.

- **[T1]** [Texas Instruments — DRV8316C, hoja de datos](https://www.ti.com/lit/ds/symlink/drv8316c.pdf). Consultar pinout, condiciones recomendadas, tabla de auxiliares, modos PWM, current sense, DRVOFF, SPI, alimentación y layout. [Página de producto](https://www.ti.com/product/DRV8316C).
- **[E1]** [Espressif — ESP32-C6 Datasheet](https://documentation.espressif.com/esp32-c6_datasheet_en.html). GPIO, ADC, MCPWM, memoria y restricciones eléctricas.
- **[E2]** [Espressif — SPI Master del ESP32-C6](https://docs.espressif.com/projects/esp-idf/en/v5.1/esp32c6/api-reference/peripherals/spi_master.html). Contrastar con la versión efectiva del SDK/HAL.
- **[E3]** [Espressif — MCPWM del ESP32-C6](https://docs.espressif.com/projects/esp-idf/en/stable/esp32c6/api-reference/peripherals/mcpwm.html). No implica que cualquier biblioteca Rust exponga todas las funciones.
- **[E4]** [Espressif — ESP32-C6-DevKitC-1](https://docs.espressif.com/projects/esp-dev-kits/en/latest/esp32c6/esp32-c6-devkitc-1/user_guide.html). Placa candidata; comprobar esquema y revisión exactos.
- **[M1]** [MagnTek — MT6701 Rev. 1.8](https://www.magntek.com.cn/upload/pdf/202407/MT6701_Rev.1.8.pdf). SSI, niveles, estados, CRC y requisitos magnéticos.
- **[L1]** `README.md` y `firmware/src/peripherals/encoder/` del repositorio. Implementación existente como contexto, no validación del nuevo motor.

No reutilizar como especificaciones las afirmaciones comerciales «100 KV», «5 A máximo», «12 V/4S» o «sin dientes» sin identificar variante y condiciones. Ningún motor concreto está homologado por este documento.
