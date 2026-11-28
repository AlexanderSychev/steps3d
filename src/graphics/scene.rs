//! Преобразования объектов в трехмерном пространстве

use super::camera::{ClippingPlanes, FieldOfView, PerspectiveCamera};
use super::light::LightSource;
use super::mesh::Mesh;
use super::transform::Transform;
use glam::Mat4;
use glam::camera::lh::view::look_to_mat4;
use palette::Srgb;
use simple_error::{SimpleResult, simple_error};
use slotmap::{SlotMap, new_key_type};
use std::collections::BTreeSet;

new_key_type! {
    pub struct NodeId;
    pub struct MeshId;
    pub struct CameraId;
    pub struct LightId;
}

/// Содежримое узла сцены
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SceneNodeContent {
    /// Пустой узел. Для потомков будет выполнять роль "опорной точки"
    Pivot,
    /// Геометрический примитив
    Mesh(MeshId),
    /// Камера
    Camera(CameraId),
    /// Источник света
    Light(LightId),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SceneNode {
    /// Локальная трансформация относительно родителя (или мира, если родителя нет)
    pub local_transform: Transform,
    /// Кэшированная мировая матрица (обновляется через Scene::update_world_matrices)
    world_matrix: Mat4,
    /// Что именно находится в этом узле
    content: SceneNodeContent,
    /// Родительский узел (если есть)
    parent: Option<NodeId>,
    /// Дочерние узлы
    children: BTreeSet<NodeId>,
}
impl Default for SceneNode {
    fn default() -> Self {
        Self {
            local_transform: Transform::default(),
            world_matrix: Mat4::IDENTITY,
            content: SceneNodeContent::Pivot,
            parent: None,
            children: BTreeSet::new(),
        }
    }
}
impl SceneNode {
    /// Получить содержимое узла
    pub fn get_content(&self) -> SceneNodeContent {
        self.content
    }

    /// Получить родительский узел
    pub fn get_parent(&self) -> Option<NodeId> {
        self.parent
    }

    /// Получить дочерние узлы
    pub fn get_children(&self) -> &BTreeSet<NodeId> {
        &self.children
    }

    /// Получить актуальную мировую матрицу узла (Model Matrix)
    pub fn get_world_matrix(&self) -> Mat4 {
        self.world_matrix
    }
}

#[derive(Debug, Default)]
pub struct Scene {
    /// Узлы сцены
    nodes: SlotMap<NodeId, SceneNode>,
    /// Геометрические объекты сцены
    meshes: SlotMap<MeshId, Mesh>,
    /// Камеры сцены
    cameras: SlotMap<CameraId, PerspectiveCamera>,
    /// Источники света
    lights: SlotMap<LightId, LightSource>,
}
impl Scene {
    /// Пересчитать мировые матрицы для всех узлов сцены (вызывать перед рендерингом кадра)
    pub fn update_world_matrices(&mut self) {
        // 1. Собираем все корневые узлы (у которых нет родителя)
        let root_ids: Vec<NodeId> = self
            .nodes
            .iter()
            .filter_map(|(id, node)| {
                if node.parent.is_none() {
                    Some(id)
                } else {
                    None
                }
            })
            .collect();

        // 2. Рекурсивно обновляем каждое поддерево
        for root_id in root_ids {
            self.update_node_recursive(root_id, Mat4::IDENTITY);
        }
    }

    /// Рекурсивное обновление мировой матрицы узла и его потомков
    fn update_node_recursive(&mut self, node_id: NodeId, parent_world: Mat4) {
        let (world_matrix, children) = {
            let Some(node) = self.nodes.get_mut(node_id) else {
                return;
            };
            let world = parent_world * node.local_transform.get_model_matrix();
            node.world_matrix = world;

            // Клонируем список детей, чтобы освободить мутабельную ссылку на self.nodes
            let children: Vec<NodeId> = node.children.iter().copied().collect();
            (world, children)
        };

        for child_id in children {
            self.update_node_recursive(child_id, world_matrix);
        }
    }

    /// Получить мировую матрицу конкретного узла (для передачи как Model Matrix в шейдер)
    pub fn get_world_matrix(&self, node_id: NodeId) -> Option<Mat4> {
        self.nodes.get(node_id).map(|node| node.world_matrix)
    }

    /// Получить матрицу вида (View Matrix) для узла с камерой
    pub fn get_camera_view_matrix(&self, camera_node_id: NodeId) -> SimpleResult<Mat4> {
        let node = self
            .nodes
            .get(camera_node_id)
            .ok_or_else(|| simple_error!("Camera node does not exist"))?;

        if !matches!(node.content, SceneNodeContent::Camera(_)) {
            return Err(simple_error!("Specified node is not a camera"));
        }

        // Извлекаем мировую позицию и мировые оси камеры из её мировой матрицы.
        // Важно: нормализуем векторы на случай, если родитель камеры был отмасштабирован!
        let world_pos = node.world_matrix.w_axis.truncate();
        let world_up = node.world_matrix.y_axis.truncate().normalize();
        let world_view = node.world_matrix.z_axis.truncate().normalize();

        Ok(look_to_mat4(world_pos, world_view, world_up))
    }

    /// Получить матрицу проекции (Projection Matrix) для узла с камерой
    pub fn get_camera_projection_matrix(&self, camera_node_id: NodeId) -> SimpleResult<Mat4> {
        let node = self
            .nodes
            .get(camera_node_id)
            .ok_or_else(|| simple_error!("Camera node does not exist"))?;

        match node.content {
            SceneNodeContent::Camera(camera_id) => {
                let camera = self
                    .cameras
                    .get(camera_id)
                    .ok_or_else(|| simple_error!("Camera data missing"))?;
                Ok(camera.get_projection_matrix())
            }
            _ => Err(simple_error!("Specified node is not a camera")),
        }
    }

    // --- Создание узлов ---

    /// Создать пустой узел (опорную точку)
    pub fn create_pivot(&mut self) -> NodeId {
        self.nodes.insert(SceneNode::default())
    }

    // --- Геометрия ---

    /// Добавить геометрию в хранилище сцены (без создания узла)
    pub fn register_mesh(&mut self, mesh: Mesh) -> MeshId {
        self.meshes.insert(mesh)
    }

    /// Получить геометрию по дескриптору для чтения
    pub fn get_mesh(&self, mesh_id: MeshId) -> Option<&Mesh> {
        self.meshes.get(mesh_id)
    }

    /// Получить геометрию по дескриптору для записи
    pub fn get_mesh_mut(&mut self, mesh_id: MeshId) -> Option<&mut Mesh> {
        self.meshes.get_mut(mesh_id)
    }

    /// Создать узел сцены с указанным мешем
    pub fn create_mesh_node(&mut self, mesh_id: MeshId) -> SimpleResult<NodeId> {
        if !self.meshes.contains_key(mesh_id) {
            return Err(simple_error!("Mesh with given MeshId does not exist"));
        }
        Ok(self.nodes.insert(SceneNode {
            content: SceneNodeContent::Mesh(mesh_id),
            ..Default::default()
        }))
    }

    // -- Камеры --

    /// Добавить камеру в сцену и вернуть ее ключ-дескриптор
    pub fn create_camera(
        &mut self,
        field_of_view: FieldOfView,
        clipping: ClippingPlanes,
    ) -> NodeId {
        let camera_id = self
            .cameras
            .insert(PerspectiveCamera::new(field_of_view, clipping));
        self.nodes.insert(SceneNode {
            content: SceneNodeContent::Camera(camera_id),
            ..Default::default()
        })
    }

    /// Получить камеру для чтения
    pub fn get_camera(&self, camera_id: CameraId) -> Option<&PerspectiveCamera> {
        self.cameras.get(camera_id)
    }

    /// Получить камеру для записи
    pub fn get_camera_mut(&mut self, camera_id: CameraId) -> Option<&mut PerspectiveCamera> {
        self.cameras.get_mut(camera_id)
    }

    // -- Источники освещения --

    /// Добавить точечный источник света в сцену и вернуть его ключ-дескриптор
    pub fn create_point_light(
        &mut self,
        intensity: f32,
        range: f32,
        color: Srgb<u8>,
    ) -> SimpleResult<NodeId> {
        let light = LightSource::try_new_point(intensity, range, color)?;
        let light_id = self.lights.insert(light);
        let node_id = self.nodes.insert(SceneNode {
            content: SceneNodeContent::Light(light_id),
            ..Default::default()
        });
        Ok(node_id)
    }

    /// Добавить направленный источник света в сцену и вернуть его ключ-дескриптор
    pub fn create_directional_light(
        &mut self,
        intensity: f32,
        color: Srgb<u8>,
    ) -> SimpleResult<NodeId> {
        let light = LightSource::try_new_directional(intensity, color)?;
        let light_id = self.lights.insert(light);
        let node_id = self.nodes.insert(SceneNode {
            content: SceneNodeContent::Light(light_id),
            ..Default::default()
        });
        Ok(node_id)
    }

    /// Добавить прожектор в сцену и вернуть его ключ-дескриптор
    pub fn create_spot_light(
        &mut self,
        intensity: f32,
        inner_angle: f32,
        outer_angle: f32,
        range: f32,
        color: Srgb<u8>,
    ) -> SimpleResult<NodeId> {
        let light = LightSource::try_new_spot(intensity, inner_angle, outer_angle, range, color)?;
        let light_id = self.lights.insert(light);
        let node_id = self.nodes.insert(SceneNode {
            content: SceneNodeContent::Light(light_id),
            ..Default::default()
        });
        Ok(node_id)
    }

    /// Получить источник света для чтения
    pub fn get_light(&self, light_id: LightId) -> Option<&LightSource> {
        self.lights.get(light_id)
    }

    /// Получить источник света для записи
    pub fn get_light_mut(&mut self, light_id: LightId) -> Option<&mut LightSource> {
        self.lights.get_mut(light_id)
    }

    // --- Доступ к узлам ---

    /// Получить узел сцены для чтения
    pub fn get_node(&self, id: NodeId) -> Option<&SceneNode> {
        self.nodes.get(id)
    }

    /// Получить узел сцены для записи
    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut SceneNode> {
        self.nodes.get_mut(id)
    }

    /// Перебрать узлы сцены вместе с их дескрипторами
    pub fn iter(&self) -> impl Iterator<Item = (NodeId, &SceneNode)> {
        self.nodes.iter()
    }

    /// Перебрать узлы сцены вместе с их дескрипторами для изменения
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (NodeId, &mut SceneNode)> {
        self.nodes.iter_mut()
    }

    // --- Управление иерархией (аналог EntityParent) ---

    /// Привязать узел `child_id` к новому родителю `new_parent_id` (или отвязать, передав `None`).
    pub fn set_parent(
        &mut self,
        child_id: NodeId,
        new_parent_id: Option<NodeId>,
    ) -> SimpleResult<()> {
        // 1. Проверяем существование дочернего узла
        if !self.nodes.contains_key(child_id) {
            return Err(simple_error!("Child node does not exist"));
        }

        // 2. Если задан новый родитель — проверяем его существование и отсутствие циклов
        if let Some(parent_id) = new_parent_id {
            if !self.nodes.contains_key(parent_id) {
                return Err(simple_error!("Parent node does not exist"));
            }
            if child_id == parent_id {
                return Err(simple_error!("Node cannot be a parent of itself"));
            }
            if self.is_ancestor(child_id, parent_id) {
                return Err(simple_error!(
                    "Cyclic hierarchy detected: target parent is a descendant of child"
                ));
            }
        }

        // 3. Отвязываем от старого родителя (если он был)
        let old_parent = self.nodes.get_mut(child_id).unwrap().parent.take();
        if let Some(old_parent_id) = old_parent {
            if let Some(old_parent_node) = self.nodes.get_mut(old_parent_id) {
                old_parent_node.children.remove(&child_id);
            }
        }

        // 4. Привязываем к новому родителю (если задан)
        if let Some(parent_id) = new_parent_id {
            self.nodes.get_mut(child_id).unwrap().parent = Some(parent_id);
            self.nodes
                .get_mut(parent_id)
                .unwrap()
                .children
                .insert(child_id);
        }

        Ok(())
    }

    /// Проверяет, является ли `candidate_ancestor` предком для `node_id`
    fn is_ancestor(&self, candidate_ancestor: NodeId, mut current_id: NodeId) -> bool {
        while let Some(node) = self.nodes.get(current_id) {
            match node.parent {
                Some(parent_id) if parent_id == candidate_ancestor => return true,
                Some(parent_id) => current_id = parent_id,
                None => return false,
            }
        }
        false
    }

    // --- Удаление узлов ---

    /// Удалить узел вместе со всем его поддеревом потомков
    pub fn remove_node(&mut self, node_id: NodeId) -> SimpleResult<()> {
        let node = self
            .nodes
            .remove(node_id)
            .ok_or_else(|| simple_error!("Node does not exist"))?;

        // Отвязываем удаляемый узел от его родителя
        if let Some(parent_id) = node.parent {
            if let Some(parent_node) = self.nodes.get_mut(parent_id) {
                parent_node.children.remove(&node_id);
            }
        }

        // Очищаем специфичные ресурсы узла (камеру; меш не трогаем, т.к. он может использоваться другими узлами)
        if let SceneNodeContent::Camera(camera_id) = node.content {
            self.cameras.remove(camera_id);
        }
        if let SceneNodeContent::Light(light_id) = node.content {
            self.lights.remove(light_id);
        }

        // Рекурсивно удаляем всех потомков
        for child_id in node.children {
            let _ = self.remove_node(child_id);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Scene;
    use glam::Vec3;

    #[test]
    fn iteration_exposes_nodes_and_allows_mutation() {
        let mut scene = Scene::default();
        let first = scene.create_pivot();
        let second = scene.create_pivot();

        let node_ids: std::collections::BTreeSet<_> =
            scene.iter().map(|(node_id, _)| node_id).collect();
        assert_eq!(node_ids, [first, second].into_iter().collect());

        for (node_id, node) in scene.iter_mut() {
            let x = if node_id == first { 1.0 } else { 2.0 };
            node.local_transform
                .set_position(Vec3::new(x, 0.0, 0.0))
                .unwrap();
        }

        assert_eq!(
            scene
                .get_node(first)
                .unwrap()
                .local_transform
                .get_position(),
            Vec3::X
        );
        assert_eq!(
            scene
                .get_node(second)
                .unwrap()
                .local_transform
                .get_position(),
            Vec3::X * 2.0
        );
    }
}
