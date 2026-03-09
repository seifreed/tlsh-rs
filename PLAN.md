# Plan

1. Analizar la implementación canónica upstream de TLSH y trasladar el algoritmo con fidelidad a Rust.
2. Implementar el núcleo del algoritmo en Rust puro:
   ventana deslizante, Pearson mapping, cuartiles, compresión de buckets, encoding, parsing y diff.
3. Diseñar una API pública limpia:
   `TlshBuilder`, `TlshDigest`, `TlshProfile`, `hash_bytes`, `hash_bytes_with_profile`, errores tipados.
4. Extender la implementación a perfiles canónicos adicionales:
   `128/3`, `256/1` y `256/3`.
5. Añadir una CLI mínima en Rust para `hash`, `hash-many`, `diff` y `xref` sobre archivos o digests.
6. Añadir exportación SARIF 2.1.0 para resultados de similitud.
7. Añadir soporte de `stdin` y salida `json` para interoperabilidad.
8. Verificar compatibilidad con vectores obtenidos del upstream local y con roundtrip de digests.

Estado: completado en esta iteración para `128/1`, `128/3`, `256/1`, `256/3`, CLI batch básica, salida SARIF y soporte `stdin/json`.

9. Refactorizar la CLI hacia módulos explícitos de arquitectura:
   `cli::args`, `cli::application`, `cli::io`, `cli::presentation` y `cli::model`.
10. Mantener el binario como adaptador fino y cubrir ramas negativas y de ayuda sin mocks.
11. Medir cobertura con `cargo llvm-cov` y cerrar los huecos hasta cobertura completa real.

Estado actual: en curso para la refactorización de arquitectura y la ampliación de cobertura al 100%.
