-- "Who had this copy" is the label mechanism with four more discriminators:
-- same free text, same suggestion pool, no second table.
ALTER TYPE "label_kind" ADD VALUE IF NOT EXISTS 'received_from';
ALTER TYPE "label_kind" ADD VALUE IF NOT EXISTS 'given_to';
ALTER TYPE "label_kind" ADD VALUE IF NOT EXISTS 'borrowed_from';
ALTER TYPE "label_kind" ADD VALUE IF NOT EXISTS 'borrowed_to';
