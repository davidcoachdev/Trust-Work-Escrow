/**
 * Trust Work Escrow — Creador de Encuesta de Validación (Milestone 4)
 * ----------------------------------------------------------------
 * Cómo usar:
 *   1. Ir a https://script.google.com
 *   2. Nuevo proyecto -> pegar TODO este código
 *   3. Guardar -> Ejecutar función "crearEncuestaTWE"
 *   4. La primera vez pide permiso (autorizar Google Forms + Sheets)
 *   5. Al terminar, el log muestra el LINK del formulario y de la hoja de respuestas
 *
 * Resultado: Google Form con 26 preguntas (bloques A freelancer / B cliente / C árbitro)
 *            + Sheet enlazado para volcar la ponderación del archivo 03.
 */

function crearEncuestaTWE() {
  const FORM_TITLE = 'Trust Work Escrow — Encuesta de Validación';
  const FORM_DESC =
    'Nos ayudás a validar un escrow descentralizado en Solana para freelancers. ' +
    'Tarda 3-4 minutos. Las respuestas son confidenciales y solo para validación de producto.';

  const form = FormApp.create(FORM_TITLE);
  form.setDescription(FORM_DESC);
  form.setCollectEmail(false);
  form.setLimitOneResponsePerUser(false);
  form.setAllowResponseEdits(true);

  // ---- Q1: Filtro de rol ----
  form.addMultipleChoiceItem()
    .setTitle('Q1. ¿Cuál es tu rol principal?')
    .setChoiceValues(['Freelancer (vendo servicios)', 'Cliente (contrato freelancers)', 'Ambos', 'Ninguno / solo curioso'])
    .setRequired(true);

  // ================= BLOQUE A — FREELANCERS =================
  form.addPageBreakItem().setTitle('Bloque A — Freelancers').setHelpText('Si vendés servicios, completá este bloque.');

  form.addTextItem().setTitle('Q2. ¿Qué hacés y cómo conseguís tus clientes hoy?').setRequired(false);

  form.addListItem()
    .setTitle('Q3. ¿Hace cuánto trabajás como freelancer?')
    .setChoiceValues(['Menos de 1 año', '1-3 años', '3-5 años', '5+ años'])
    .setRequired(false);

  form.addTextItem().setTitle('Q4. ¿Qué es lo que más te frustra de plataformas como Upwork, Fiverr o LaborX?').setRequired(false);

  form.addTextItem().setTitle('Q5. ¿Cuánto tarda el pago tras entregar y cuánto descuenta la plataforma?').setRequired(false);

  form.addMultipleChoiceItem()
    .setTitle('Q6. ¿Alguna vez tuviste un problema o disputa con un pago?')
    .setChoiceValues(['Sí', 'No'])
    .setRequired(false);

  form.addTextItem().setTitle('Q7. (Si marcaste Sí) ¿Cómo se resolvió?').setRequired(false);

  // Q8: ranking como 4 dropdowns de importancia (1=mayor, 4=menor)
  const factores = ['Comisiones bajas', 'Velocidad de pago', 'Confianza / seguridad', 'Resolución justa de disputas'];
  factores.forEach(function (f) {
    form.addListItem()
      .setTitle('Q8. ¿Qué importancia le das a "' + f + '" al elegir plataforma? (1 = mayor, 4 = menor)')
      .setChoiceValues(['1 — Más importante', '2', '3', '4 — Menos importante'])
      .setRequired(false);
  });

  form.addTextItem().setTitle('Q9. ¿Por qué pusiste primero a ese factor?').setRequired(false);

  form.addMultipleChoiceItem()
    .setTitle('Q10. ¿Usarías un sistema con escrow on-chain en Solana, comisión <5% y pago inmediato al aprobar?')
    .setChoiceValues(['Sí', 'No', 'Quizás'])
    .setRequired(false);

  form.addMultipleChoiceItem()
    .setTitle('Q11. Si ya pagás 20% en Upwork, ¿el ahorro solo te haría cambiar, o necesitás algo más?')
    .setChoiceValues([
      'A) Solo el ahorro me alcanza para cambiar',
      'B) Necesito confianza/garantía, el ahorro no alcanza',
      'C) El ahorro me llama, pero no migro sin confianza'
    ])
    .setRequired(false);

  form.addTextItem().setTitle('Q12. ¿Qué te haría cambiar desde Upwork/Fiverr a Trust Work Escrow mañana?').setRequired(false);

  form.addCheckboxItem()
    .setTitle('Q13. ¿Qué te generaría confianza para meter tu primera plata en una plataforma nueva?')
    .setChoiceValues([
      'Reputación verificable',
      'Contrato auditado / open source',
      'Conocidos que ya la usen',
      'Garantía de disputa / árbitros',
      'Otra'
    ])
    .setRequired(false);

  form.addMultipleChoiceItem()
    .setTitle('Q14. ¿Te da más confianza ver fondos bloqueados en un explorador on-chain que el sistema de reputación de Upwork?')
    .setChoiceValues(['Sí', 'No', 'No sé'])
    .setRequired(false);

  form.addTextItem().setTitle('Q15. ¿Hay algo más que quieras agregar?').setRequired(false);

  form.addTextItem().setTitle('Q16. ¿Te puedo contactar para probar la versión? (email o Discord, opcional)').setRequired(false);

  // ================= BLOQUE B — CLIENTES =================
  form.addPageBreakItem().setTitle('Bloque B — Clientes').setHelpText('Si contratás freelancers, completá este bloque.');

  form.addTextItem().setTitle('Q17. ¿Cómo encontrás y contratás freelancers hoy?').setRequired(false);

  form.addTextItem().setTitle('Q18. ¿Cuántos contratás por mes y qué presupuesto manejás?').setRequired(false);

  form.addTextItem().setTitle('Q19. ¿Qué es lo que menos te gusta del proceso de pago actual?').setRequired(false);

  form.addTextItem().setTitle('Q20. ¿Alguna vez perdiste plata con un freelancer? ¿Cómo lo manejaste?').setRequired(false);

  form.addTextItem().setTitle('Q21. ¿Cómo generás confianza con alguien que contratás por primera vez?').setRequired(false);

  form.addScaleItem()
    .setTitle('Q22. ¿Qué opinás de bloquear el pago al inicio y liberarlo solo al aprobar el trabajo?')
    .setBounds(1, 5)
    .setLabels('Muy mala', 'Muy buena')
    .setRequired(false);

  form.addMultipleChoiceItem()
    .setTitle('Q23. ¿Te daría más confianza ver los fondos en el explorador de Solana?')
    .setChoiceValues(['Sí', 'No'])
    .setRequired(false);

  form.addMultipleChoiceItem()
    .setTitle('Q24. ¿Cambiarías a una plataforma sin comisiones aunque uses wallet crypto?')
    .setChoiceValues(['Sí', 'No', 'Quizás'])
    .setRequired(false);

  // ================= BLOQUE C — ÁRBITROS =================
  form.addPageBreakItem().setTitle('Bloque C — Árbitros (opcional)').setHelpText('Si te interesa resolver disputas, completá esto.');

  form.addMultipleChoiceItem()
    .setTitle('Q25. ¿Tenés experiencia resolviendo disputas freelance?')
    .setChoiceValues(['Sí', 'No'])
    .setRequired(false);

  form.addTextItem().setTitle('Q26. ¿Participarías en un sistema de justicia descentralizado? ¿Qué incentivo necesitarías?').setRequired(false);

  // ---- Enlazar a Sheets para análisis ----
  const ss = SpreadsheetApp.create(FORM_TITLE + ' — Respuestas');
  form.setDestination(FormApp.DestinationType.SPREADSHEET, ss.getId());

  const url = form.getEditUrl();
  const responseUrl = form.getPublishedUrl();
  Logger.log('FORMULARIO (editar): ' + url);
  Logger.log('FORMULARIO (responder): ' + responseUrl);
  Logger.log('HOJA DE RESPUESTAS: ' + ss.getUrl());

  return { formUrl: responseUrl, sheetUrl: ss.getUrl() };
}
