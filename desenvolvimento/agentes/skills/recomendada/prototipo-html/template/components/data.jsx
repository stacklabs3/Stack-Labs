// Protótipo — Mock data e defaults para o prototipo
// TEMPLATE: adicione dados mockados (listas, registros, configs) que a tela consome.
// IMPORTANTE: dados ficticios — nunca colocar PII ou dados reais de cliente aqui.

const MOCK_USER = {
  name: 'Cliente Exemplo',
  role: 'customer',
  email: 'exemplo@dominio.local',
};

const MOCK_ITEMS = [
  { id: 1, label: 'Item exemplo 1', value: 10 },
  { id: 2, label: 'Item exemplo 2', value: 20 },
  { id: 3, label: 'Item exemplo 3', value: 30 },
];

/* Superfície e tema de abertura vêm da aba ativa, em visoes.jsx. */
const DEFAULTS = {
  estado: 'default',
  mode: 'prototype',
};
