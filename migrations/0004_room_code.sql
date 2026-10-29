-- Code court aleatoire (6 chiffres) pour rejoindre un salon, au lieu de
-- l'id interne sequentiel : un id 1, 2, 3... se devine trivialement (n'importe
-- qui peut parcourir tous les salons en incrementant), le code non.
ALTER TABLE rooms ADD COLUMN code CHAR(6);

UPDATE rooms SET code = lpad((floor(random() * 1000000))::text, 6, '0') WHERE code IS NULL;

ALTER TABLE rooms ALTER COLUMN code SET NOT NULL;
ALTER TABLE rooms ADD CONSTRAINT rooms_code_key UNIQUE (code);
