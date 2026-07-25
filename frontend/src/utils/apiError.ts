// Maps an HTTP status to a translated, user-facing message. The backend's
// English `{ "error": "..." }` prose is intentionally ignored here — it stays
// developer/log-facing while the UI owns all translated text.
// ponytail: status → message is enough while backend errors are generic; add a
// per-code protocol only when a message needs backend-only info the client lacks.
import type { ComposerTranslation } from 'vue-i18n'

export function apiErrorMessage(status: number, t: ComposerTranslation): string {
  switch (status) {
    case 400: return t('error.badRequest')
    case 401: return t('error.unauthorized')
    case 403: return t('error.forbidden')
    case 404: return t('error.notFound')
    case 409: return t('error.conflict')
    default: return t('error.server')
  }
}
