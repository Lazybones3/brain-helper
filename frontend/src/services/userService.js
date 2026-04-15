import api from "@/lib/api";

export const userService = {
  login: (data) => api.post('/user/login', data),

  // READ (Get all)
  getAll: () => api.get("/products"),

  // READ (Get one)
  getById: (id) => api.get(`/products/${id}`),

  // CREATE
  create: (data) => api.post("/products", data),

  // UPDATE
  update: (id, data) => api.put(`/products/${id}`, data),

  // DELETE
  delete: (id) => api.delete(`/products/${id}`),
};