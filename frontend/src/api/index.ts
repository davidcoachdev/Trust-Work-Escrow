/**
 * frontend/src/api — barrel raíz.
 * Re-exporta client root + todos los módulos por dominio.
 * Cada sub-carpeta contiene un archivo por endpoint/ruta.
 */
export * from "./client";
export * from "./types";
export * as jobs from "./jobs";
export * as applications from "./applications";
export * as milestones from "./milestones";
export * as disputes from "./disputes";
export * as support from "./support";
export * as arbiterPool from "./arbiterPool";
export * as config from "./config";
export * as auth from "./auth";
export * as health from "./health";
