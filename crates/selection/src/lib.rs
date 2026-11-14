//! Selection system for Creative Engine
//!
//! Provides comprehensive selection tools: click, rectangle, lasso,
//! multi-select with modifiers, and selection manipulation.

use creative_domain::{DocumentId, ObjectId, LayerId, Point2, Rect, Vector2, Transform2D};
use creative_geometry::{Geometry, BoundingBox};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Selection errors
#[derive(Debug, Error)]
pub enum SelectionError {
    #[error("No object at position")]
    NoObjectAtPosition,
    #[error("Object not found: {0}")]
    ObjectNotFound(ObjectId),
    #[error("Layer not found: {0}")]
    LayerNotFound(LayerId),
    #[error("Invalid selection state: {0}")]
    InvalidState(String),
}

/// Selection modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SelectionMode {
    /// Single click selection
    #[default]
    Click,
    /// Rectangle marquee selection
    Rectangle,
    /// Freeform lasso selection
    Lasso,
    /// Select all on layer
    AllOnLayer,
    /// Select all in document
    AllInDocument,
}

/// Selection modifiers
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectionModifiers {
    pub add: bool,       // Shift - add to selection
    pub subtract: bool,  // Alt/Option - subtract from selection
    pub toggle: bool,    // Ctrl/Cmd - toggle selection
}

/// Selection state for a document
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Selection {
    pub document_id: DocumentId,
    pub selected_objects: HashSet<ObjectId>,
    pub active_layer: Option<LayerId>,
    pub anchor_point: Option<Point2>,  // For rectangle/lasso
    pub current_point: Option<Point2>, // For rectangle/lasso
    pub mode: SelectionMode,
    pub modifiers: SelectionModifiers,
}

/// Selection manager
pub struct SelectionManager {
    selection: Selection,
    hit_test_cache: HashMap<ObjectId, BoundingBox>,
}

impl SelectionManager {
    /// Create a new selection manager
    pub fn new(document_id: DocumentId) -> Self {
        Self {
            selection: Selection {
                document_id,
                ..Default::default()
            },
            hit_test_cache: HashMap::new(),
        }
    }
    
    /// Get current selection
    pub fn selection(&self) -> &Selection {
        &self.selection
    }
    
    /// Get mutable selection
    pub fn selection_mut(&mut self) -> &mut Selection {
        &mut self.selection
    }
    
    /// Set selection mode
    pub fn set_mode(&mut self, mode: SelectionMode) {
        self.selection.mode = mode;
    }
    
    /// Set selection modifiers
    pub fn set_modifiers(&mut self, modifiers: SelectionModifiers) {
        self.selection.modifiers = modifiers;
    }
    
    /// Update hit test cache with object bounding boxes
    pub fn update_hit_test_cache(&mut self, objects: &HashMap<ObjectId, (BoundingBox, Transform2D)>) {
        self.hit_test_cache.clear();
        for (id, (bbox, _transform)) in objects {
            self.hit_test_cache.insert(*id, *bbox);
        }
    }
    
    /// Hit test a point against objects (returns topmost object)
    pub fn hit_test_point(&self, point: Point2, objects: &HashMap<ObjectId, (BoundingBox, Transform2D)>) -> Option<ObjectId> {
        // Check objects in reverse z-order (topmost first)
        let mut candidates: Vec<_> = objects.iter()
            .filter(|(id, (bbox, transform))| {
                // Transform point to object space
                let inv_transform = transform.inverse();
                if let Some(inv) = inv_transform {
                    let local_point = inv.transform_point(point);
                    bbox.contains(local_point)
                } else {
                    false
                }
            })
            .collect();
        
        // Sort by z-index (would need z-index in the map, simplified for now)
        // Using UUID as a stable but arbitrary ordering
        candidates.sort_by(|a, b| a.0.0.cmp(&b.0.0));
        
        candidates.first().map(|(id, _)| **id)
    }
    
    /// Hit test rectangle against objects
    pub fn hit_test_rect(&self, rect: Rect, objects: &HashMap<ObjectId, (BoundingBox, Transform2D)>) -> Vec<ObjectId> {
        objects.iter()
            .filter(|(id, (bbox, transform))| {
                // Quick bbox check in world space
                let world_bbox = self.transform_bbox(bbox, transform);
                world_bbox.intersects(BoundingBox::new(rect.min, rect.max))
            })
            .map(|(id, _)| *id)
            .collect()
    }
    
    /// Hit test lasso (polygon) against objects
    pub fn hit_test_lasso(&self, points: &[Point2], objects: &HashMap<ObjectId, (BoundingBox, Transform2D)>) -> Vec<ObjectId> {
        if points.len() < 3 {
            return Vec::new();
        }
        
        objects.iter()
            .filter(|(id, (bbox, transform))| {
                let world_bbox = self.transform_bbox(bbox, transform);
                // Check if any corner of bbox is inside lasso
                let corners = [
                    Point2::new(world_bbox.min.x, world_bbox.min.y),
                    Point2::new(world_bbox.max.x, world_bbox.min.y),
                    Point2::new(world_bbox.max.x, world_bbox.max.y),
                    Point2::new(world_bbox.min.x, world_bbox.max.y),
                ];
                
                corners.iter().any(|corner| self.point_in_polygon(corner, points))
            })
            .map(|(id, _)| *id)
            .collect()
    }
    
    /// Transform bbox by transform
    fn transform_bbox(&self, bbox: &BoundingBox, transform: &Transform2D) -> BoundingBox {
        let corners = [
            Point2::new(bbox.min.x, bbox.min.y),
            Point2::new(bbox.max.x, bbox.min.y),
            Point2::new(bbox.max.x, bbox.max.y),
            Point2::new(bbox.min.x, bbox.max.y),
        ];
        
        let transformed: Vec<Point2> = corners.iter()
            .map(|c| transform.transform_point(*c))
            .collect();
        
        let min_x = transformed.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
        let max_x = transformed.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max);
        let min_y = transformed.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
        let max_y = transformed.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max);
        
        BoundingBox::new(
            Point2::new(min_x, min_y),
            Point2::new(max_x, max_y),
        )
    }
    
    /// Point in polygon test (ray casting)
    fn point_in_polygon(&self, point: &Point2, polygon: &[Point2]) -> bool {
        let mut inside = false;
        let mut j = polygon.len() - 1;
        
        for i in 0..polygon.len() {
            let pi = &polygon[i];
            let pj = &polygon[j];
            
            if ((pi.y > point.y) != (pj.y > point.y)) &&
               (point.x < (pj.x - pi.x) * (point.y - pi.y) / (pj.y - pi.y) + pi.x) {
                inside = !inside;
            }
            j = i;
        }
        
        inside
    }
    
    /// Begin selection operation
    pub fn begin_selection(&mut self, screen_point: Point2, mode: SelectionMode) {
        self.selection.mode = mode;
        self.selection.anchor_point = Some(screen_point);
        self.selection.current_point = Some(screen_point);
    }
    
    /// Update selection operation
    pub fn update_selection(&mut self, screen_point: Point2) {
        self.selection.current_point = Some(screen_point);
    }
    
    /// End selection operation and apply
    pub fn end_selection(
        &mut self,
        objects: &HashMap<ObjectId, (BoundingBox, Transform2D)>,
    ) -> Vec<ObjectId> {
        let selected = match self.selection.mode {
            SelectionMode::Click => {
                if let Some(anchor) = self.selection.anchor_point {
                    self.hit_test_point(anchor, objects).into_iter().collect()
                } else {
                    Vec::new()
                }
            }
            SelectionMode::Rectangle => {
                if let (Some(anchor), Some(current)) = (self.selection.anchor_point, self.selection.current_point) {
                    let rect = Rect::new(anchor, current);
                    self.hit_test_rect(rect, objects)
                } else {
                    Vec::new()
                }
            }
            SelectionMode::Lasso => {
                // For lasso, we'd need the full path - simplified here
                Vec::new()
            }
            SelectionMode::AllOnLayer => {
                if let Some(layer) = self.selection.active_layer {
                    objects.iter()
                        .filter(|(_, (_, _))| true) // Would need layer info
                        .map(|(id, _)| *id)
                        .collect()
                } else {
                    Vec::new()
                }
            }
            SelectionMode::AllInDocument => {
                objects.keys().copied().collect()
            }
        };
        
        self.apply_selection(selected);
        
        // Reset anchor points
        self.selection.anchor_point = None;
        self.selection.current_point = None;
        
        self.selection.selected_objects.iter().copied().collect()
    }
    
    /// Apply selection with modifiers
    fn apply_selection(&mut self, new_selection: Vec<ObjectId>) {
        let new_set: HashSet<_> = new_selection.into_iter().collect();
        
        match (self.selection.modifiers.add, self.selection.modifiers.subtract, self.selection.modifiers.toggle) {
            (true, false, false) => {
                // Add to selection
                self.selection.selected_objects.extend(new_set);
            }
            (false, true, false) => {
                // Subtract from selection
                for id in new_set {
                    self.selection.selected_objects.remove(&id);
                }
            }
            (false, false, true) => {
                // Toggle selection
                for id in new_set {
                    if self.selection.selected_objects.contains(&id) {
                        self.selection.selected_objects.remove(&id);
                    } else {
                        self.selection.selected_objects.insert(id);
                    }
                }
            }
            _ => {
                // Replace selection
                self.selection.selected_objects = new_set;
            }
        }
    }
    
    /// Select specific objects
    pub fn select_objects(&mut self, object_ids: Vec<ObjectId>, add_to_selection: bool) {
        let new_set: HashSet<_> = object_ids.into_iter().collect();
        
        if add_to_selection {
            self.selection.selected_objects.extend(new_set);
        } else {
            self.selection.selected_objects = new_set;
        }
    }
    
    /// Deselect all
    pub fn deselect_all(&mut self) {
        self.selection.selected_objects.clear();
    }
    
    /// Select all objects
    pub fn select_all(&mut self, objects: &HashMap<ObjectId, (BoundingBox, Transform2D)>) {
        self.selection.selected_objects = objects.keys().copied().collect();
    }
    
    /// Invert selection
    pub fn invert_selection(&mut self, objects: &HashMap<ObjectId, (BoundingBox, Transform2D)>) {
        let all_objects: HashSet<_> = objects.keys().copied().collect();
        let current = std::mem::take(&mut self.selection.selected_objects);
        self.selection.selected_objects = all_objects.difference(&current).copied().collect();
    }
    
    /// Get selected object count
    pub fn selected_count(&self) -> usize {
        self.selection.selected_objects.len()
    }
    
    /// Check if object is selected
    pub fn is_selected(&self, object_id: ObjectId) -> bool {
        self.selection.selected_objects.contains(&object_id)
    }
    
    /// Get selected object IDs
    pub fn selected_ids(&self) -> Vec<ObjectId> {
        self.selection.selected_objects.iter().copied().collect()
    }
    
    /// Set active layer
    pub fn set_active_layer(&mut self, layer_id: Option<LayerId>) {
        self.selection.active_layer = layer_id;
    }
    
    /// Get active layer
    pub fn active_layer(&self) -> Option<LayerId> {
        self.selection.active_layer
    }
    
    /// Get selection bounds (union of selected object bounds)
    pub fn selection_bounds(&self, objects: &HashMap<ObjectId, (BoundingBox, Transform2D)>) -> Option<BoundingBox> {
        let mut bounds: Option<BoundingBox> = None;
        
        for id in &self.selection.selected_objects {
            if let Some((bbox, transform)) = objects.get(id) {
                let world_bbox = self.transform_bbox(bbox, transform);
                bounds = Some(match bounds {
                    Some(b) => b.union(world_bbox),
                    None => world_bbox,
                });
            }
        }
        
        bounds
    }
    
    /// Get selection center
    pub fn selection_center(&self, objects: &HashMap<ObjectId, (BoundingBox, Transform2D)>) -> Option<Point2> {
        self.selection_bounds(objects).map(|b| b.center())
    }
    
    /// Transform selection (move, rotate, scale)
    pub fn transform_selection(
        &mut self,
        transform: Transform2D,
        pivot: Option<Point2>,
        objects: &mut HashMap<ObjectId, (BoundingBox, Transform2D)>,
    ) -> Vec<ObjectId> {
        let pivot = pivot.or_else(|| self.selection_center(objects)).unwrap_or(Point2::ZERO);
        let mut transformed = Vec::new();
        
        for id in &self.selection.selected_objects {
            if let Some((bbox, obj_transform)) = objects.get_mut(id) {
                // Apply transform around pivot
                let to_pivot = Transform2D::translate(-pivot.x, -pivot.y);
                let from_pivot = Transform2D::translate(pivot.x, pivot.y);
                *obj_transform = from_pivot * transform * to_pivot * *obj_transform;
                
                // Update bbox
                *bbox = self.transform_bbox(bbox, &transform);
                transformed.push(*id);
            }
        }
        
        transformed
    }
    
    /// Get selection rectangle for drawing (marquee)
    pub fn selection_rect(&self) -> Option<Rect> {
        match (self.selection.anchor_point, self.selection.current_point) {
            (Some(anchor), Some(current)) => Some(Rect::new(anchor, current)),
            _ => None,
        }
    }
    
    /// Get lasso points for drawing
    pub fn lasso_points(&self) -> Option<Vec<Point2>> {
        // Would need to store lasso path during drag
        None
    }
}

/// Selection commands for integration with command system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SelectionCommand {
    SelectObjects {
        object_ids: Vec<ObjectId>,
        add_to_selection: bool,
    },
    SelectAll {
        layer_id: Option<LayerId>,
    },
    DeselectAll,
    SelectByRectangle {
        rect: Rect,
        add_to_selection: bool,
    },
    SelectByLasso {
        points: Vec<Point2>,
        add_to_selection: bool,
    },
    InvertSelection {
        layer_id: Option<LayerId>,
    },
    SetActiveLayer {
        layer_id: Option<LayerId>,
    },
    TransformSelection {
        transform: Transform2D,
        pivot: Option<Point2>,
    },
}

impl SelectionCommand {
    pub fn document_id(&self) -> DocumentId {
        // Would need document context - placeholder
        DocumentId(Uuid::nil())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use creative_domain::{DocumentId, ObjectId, LayerId, Point2, Rect, Transform2D, Vector2};
    use creative_geometry::{BoundingBox, GeometryId};
    use std::collections::HashMap;
    use uuid::Uuid;
    
    fn create_test_objects() -> (HashMap<ObjectId, (BoundingBox, Transform2D)>, ObjectId, ObjectId, ObjectId) {
        let mut objects = HashMap::new();
        
        // Object 1 at (0,0) size 100x100
        let obj1 = ObjectId::new();
        let bbox1 = BoundingBox::new(Point2::new(0.0, 0.0), Point2::new(100.0, 100.0));
        objects.insert(obj1, (bbox1, Transform2D::IDENTITY));
        
        // Object 2 at (200,200) size 50x50
        let obj2 = ObjectId::new();
        let bbox2 = BoundingBox::new(Point2::new(200.0, 200.0), Point2::new(250.0, 250.0));
        objects.insert(obj2, (bbox2, Transform2D::IDENTITY));
        
        // Object 3 at (400,400) size 100x100
        let obj3 = ObjectId::new();
        let bbox3 = BoundingBox::new(Point2::new(400.0, 400.0), Point2::new(500.0, 500.0));
        objects.insert(obj3, (bbox3, Transform2D::IDENTITY));
        
        (objects, obj1, obj2, obj3)
    }
    
    #[test]
    fn test_selection_manager_creation() {
        let doc_id = DocumentId::new();
        let manager = SelectionManager::new(doc_id);
        
        assert_eq!(manager.selection.document_id, doc_id);
        assert_eq!(manager.selected_count(), 0);
    }
    
    #[test]
    fn test_click_selection() {
        let doc_id = DocumentId::new();
        let mut manager = SelectionManager::new(doc_id);
        let (objects, _obj1, _obj2, _obj3) = create_test_objects();
        
        // Click on object 1
        manager.begin_selection(Point2::new(50.0, 50.0), SelectionMode::Click);
        let selected = manager.end_selection(&objects);
        
        assert_eq!(selected.len(), 1);
        assert!(manager.is_selected(selected[0]));
    }
    
    #[test]
    fn test_rectangle_selection() {
        let doc_id = DocumentId::new();
        let mut manager = SelectionManager::new(doc_id);
        let (objects, _obj1, _obj2, _obj3) = create_test_objects();
        
        // Rectangle selecting first two objects
        manager.begin_selection(Point2::new(0.0, 0.0), SelectionMode::Rectangle);
        manager.update_selection(Point2::new(300.0, 300.0));
        let selected = manager.end_selection(&objects);
        
        assert_eq!(selected.len(), 2);
    }
    
    #[test]
    fn test_add_to_selection() {
        let doc_id = DocumentId::new();
        let mut manager = SelectionManager::new(doc_id);
        let (objects, _obj1, _obj2, _obj3) = create_test_objects();
        
        // Select first object
        manager.begin_selection(Point2::new(50.0, 50.0), SelectionMode::Click);
        manager.end_selection(&objects);
        assert_eq!(manager.selected_count(), 1);
        
        // Add second object with shift
        manager.set_modifiers(SelectionModifiers { add: true, ..Default::default() });
        manager.begin_selection(Point2::new(225.0, 225.0), SelectionMode::Click);
        manager.end_selection(&objects);
        
        assert_eq!(manager.selected_count(), 2);
    }
    
    #[test]
    fn test_subtract_from_selection() {
        let doc_id = DocumentId::new();
        let mut manager = SelectionManager::new(doc_id);
        let (objects, _obj1, _obj2, _obj3) = create_test_objects();
        
        // Select all
        manager.select_all(&objects);
        assert_eq!(manager.selected_count(), 3);
        
        // Subtract one with alt
        manager.set_modifiers(SelectionModifiers { subtract: true, ..Default::default() });
        manager.begin_selection(Point2::new(50.0, 50.0), SelectionMode::Click);
        manager.end_selection(&objects);
        
        assert_eq!(manager.selected_count(), 2);
    }
    
    #[test]
    fn test_toggle_selection() {
        let doc_id = DocumentId::new();
        let mut manager = SelectionManager::new(doc_id);
        let (objects, _obj1, _obj2, _obj3) = create_test_objects();
        
        // Select first object
        manager.begin_selection(Point2::new(50.0, 50.0), SelectionMode::Click);
        manager.end_selection(&objects);
        assert!(manager.is_selected(manager.selected_ids()[0]));
        
        // Toggle same object with ctrl
        manager.set_modifiers(SelectionModifiers { toggle: true, ..Default::default() });
        manager.begin_selection(Point2::new(50.0, 50.0), SelectionMode::Click);
        manager.end_selection(&objects);
        
        assert_eq!(manager.selected_count(), 0);
    }
    
    #[test]
    fn test_deselect_all() {
        let doc_id = DocumentId::new();
        let mut manager = SelectionManager::new(doc_id);
        let (objects, _obj1, _obj2, _obj3) = create_test_objects();
        
        manager.select_all(&objects);
        assert_eq!(manager.selected_count(), 3);
        
        manager.deselect_all();
        assert_eq!(manager.selected_count(), 0);
    }
    
    #[test]
    fn test_invert_selection() {
        let doc_id = DocumentId::new();
        let mut manager = SelectionManager::new(doc_id);
        let (objects, obj1, _obj2, _obj3) = create_test_objects();
        
        // Select first object
        manager.select_objects(vec![obj1], false);
        assert_eq!(manager.selected_count(), 1);
        
        // Invert
        manager.invert_selection(&objects);
        assert_eq!(manager.selected_count(), 2);
        assert!(!manager.is_selected(obj1));
    }
    
    #[test]
    fn test_select_objects_direct() {
        let doc_id = DocumentId::new();
        let mut manager = SelectionManager::new(doc_id);
        let (objects, obj1, obj2, _obj3) = create_test_objects();
        
        let ids: Vec<_> = vec![obj1, obj2];
        manager.select_objects(ids.clone(), false);
        
        assert_eq!(manager.selected_count(), 2);
        for id in ids {
            assert!(manager.is_selected(id));
        }
    }
    
    #[test]
    fn test_selection_bounds() {
        let doc_id = DocumentId::new();
        let mut manager = SelectionManager::new(doc_id);
        let (objects, obj1, obj2, _obj3) = create_test_objects();
        
        let ids: Vec<_> = vec![obj1, obj2];
        manager.select_objects(ids, false);
        
        let bounds = manager.selection_bounds(&objects).unwrap();
        assert!((bounds.min.x - 0.0).abs() < 0.01);
        assert!((bounds.min.y - 0.0).abs() < 0.01);
        assert!((bounds.max.x - 250.0).abs() < 0.01);
        assert!((bounds.max.y - 250.0).abs() < 0.01);
    }
    
    #[test]
    fn test_selection_center() {
        let doc_id = DocumentId::new();
        let mut manager = SelectionManager::new(doc_id);
        let (objects, obj1, _obj2, _obj3) = create_test_objects();
        
        manager.select_objects(vec![obj1], false);
        
        let center = manager.selection_center(&objects).unwrap();
        assert!((center.x - 50.0).abs() < 0.01);
        assert!((center.y - 50.0).abs() < 0.01);
    }
    
    #[test]
    fn test_transform_selection() {
        let doc_id = DocumentId::new();
        let mut manager = SelectionManager::new(doc_id);
        let (mut objects, obj1, _obj2, _obj3) = create_test_objects();
        
        manager.select_objects(vec![obj1], false);
        
        // Translate by (10, 20)
        let transform = Transform2D::translate(10.0, 20.0);
        manager.transform_selection(transform, None, &mut objects);
        
        // Check object was moved
        let obj_id = manager.selected_ids()[0];
        let (bbox, _transform) = &objects[&obj_id];
        assert!((bbox.min.x - 10.0).abs() < 0.01);
        assert!((bbox.min.y - 20.0).abs() < 0.01);
    }
    
    #[test]
    fn test_selection_rect() {
        let doc_id = DocumentId::new();
        let mut manager = SelectionManager::new(doc_id);
        
        assert!(manager.selection_rect().is_none());
        
        manager.begin_selection(Point2::new(0.0, 0.0), SelectionMode::Rectangle);
        manager.update_selection(Point2::new(100.0, 100.0));
        
        let rect = manager.selection_rect().unwrap();
        assert_eq!(rect.min, Point2::new(0.0, 0.0));
        assert_eq!(rect.max, Point2::new(100.0, 100.0));
    }
}