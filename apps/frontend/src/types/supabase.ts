export type Json =
  | string
  | number
  | boolean
  | null
  | { [key: string]: Json | undefined }
  | Json[]

export type Database = {
  graphql_public: {
    Tables: {
      [_ in never]: never
    }
    Views: {
      [_ in never]: never
    }
    Functions: {
      graphql: {
        Args: {
          operationName?: string
          query?: string
          variables?: Json
          extensions?: Json
        }
        Returns: Json
      }
    }
    Enums: {
      [_ in never]: never
    }
    CompositeTypes: {
      [_ in never]: never
    }
  }
  public: {
    Tables: {
      ant_castes: {
        Row: {
          base_attributes: Json
          caste_name: string
          id: number
          maintenance_cost: Json | null
          population_cap_percentage: number | null
          special_abilities: Json | null
          specialization: string
          training_requirements: Json | null
          unlock_conditions: Json | null
        }
        Insert: {
          base_attributes: Json
          caste_name: string
          id?: number
          maintenance_cost?: Json | null
          population_cap_percentage?: number | null
          special_abilities?: Json | null
          specialization: string
          training_requirements?: Json | null
          unlock_conditions?: Json | null
        }
        Update: {
          base_attributes?: Json
          caste_name?: string
          id?: number
          maintenance_cost?: Json | null
          population_cap_percentage?: number | null
          special_abilities?: Json | null
          specialization?: string
          training_requirements?: Json | null
          unlock_conditions?: Json | null
        }
        Relationships: []
      }
      ant_genetics: {
        Row: {
          ant_id: string
          created_at: string | null
          fitness_score: number | null
          generation: number
          genes: Json
          id: string
          mutations: Json | null
          parent1_id: string | null
          parent2_id: string | null
        }
        Insert: {
          ant_id: string
          created_at?: string | null
          fitness_score?: number | null
          generation?: number
          genes: Json
          id?: string
          mutations?: Json | null
          parent1_id?: string | null
          parent2_id?: string | null
        }
        Update: {
          ant_id?: string
          created_at?: string | null
          fitness_score?: number | null
          generation?: number
          genes?: Json
          id?: string
          mutations?: Json | null
          parent1_id?: string | null
          parent2_id?: string | null
        }
        Relationships: [
          {
            foreignKeyName: "ant_genetics_ant_id_fkey"
            columns: ["ant_id"]
            isOneToOne: false
            referencedRelation: "ants"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "ant_genetics_parent1_id_fkey"
            columns: ["parent1_id"]
            isOneToOne: false
            referencedRelation: "ants"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "ant_genetics_parent2_id_fkey"
            columns: ["parent2_id"]
            isOneToOne: false
            referencedRelation: "ants"
            referencedColumns: ["id"]
          },
        ]
      }
      ant_interactions: {
        Row: {
          ant1_id: string
          ant2_id: string
          created_at: string | null
          damage_dealt: number | null
          id: string
          interaction_type: string
          outcome: string | null
          position_x: number | null
          position_y: number | null
          resources_exchanged: Json | null
          tick_occurred: number
        }
        Insert: {
          ant1_id: string
          ant2_id: string
          created_at?: string | null
          damage_dealt?: number | null
          id?: string
          interaction_type: string
          outcome?: string | null
          position_x?: number | null
          position_y?: number | null
          resources_exchanged?: Json | null
          tick_occurred: number
        }
        Update: {
          ant1_id?: string
          ant2_id?: string
          created_at?: string | null
          damage_dealt?: number | null
          id?: string
          interaction_type?: string
          outcome?: string | null
          position_x?: number | null
          position_y?: number | null
          resources_exchanged?: Json | null
          tick_occurred?: number
        }
        Relationships: [
          {
            foreignKeyName: "ant_interactions_ant1_id_fkey"
            columns: ["ant1_id"]
            isOneToOne: false
            referencedRelation: "ants"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "ant_interactions_ant2_id_fkey"
            columns: ["ant2_id"]
            isOneToOne: false
            referencedRelation: "ants"
            referencedColumns: ["id"]
          },
        ]
      }
      ant_types: {
        Row: {
          base_health: number
          base_size: number
          base_speed: number
          base_strength: number
          carrying_capacity: number
          color_hue: number
          food_preferences: Json | null
          id: number
          lifespan_ticks: number
          name: string
          role: string
          special_abilities: Json | null
        }
        Insert: {
          base_health?: number
          base_size?: number
          base_speed?: number
          base_strength?: number
          carrying_capacity?: number
          color_hue?: number
          food_preferences?: Json | null
          id?: number
          lifespan_ticks?: number
          name: string
          role: string
          special_abilities?: Json | null
        }
        Update: {
          base_health?: number
          base_size?: number
          base_speed?: number
          base_strength?: number
          carrying_capacity?: number
          color_hue?: number
          food_preferences?: Json | null
          id?: number
          lifespan_ticks?: number
          name?: string
          role?: string
          special_abilities?: Json | null
        }
        Relationships: []
      }
      ants: {
        Row: {
          age_ticks: number
          angle: number
          ant_type_id: number
          carried_resources: Json | null
          colony_id: string
          created_at: string | null
          current_speed: number
          energy: number
          health: number
          id: string
          last_updated: string | null
          mood: string | null
          position_x: number
          position_y: number
          state: string
          target_id: string | null
          target_type: string | null
          target_x: number | null
          target_y: number | null
          traits: Json | null
        }
        Insert: {
          age_ticks?: number
          angle?: number
          ant_type_id: number
          carried_resources?: Json | null
          colony_id: string
          created_at?: string | null
          current_speed: number
          energy?: number
          health: number
          id?: string
          last_updated?: string | null
          mood?: string | null
          position_x: number
          position_y: number
          state?: string
          target_id?: string | null
          target_type?: string | null
          target_x?: number | null
          target_y?: number | null
          traits?: Json | null
        }
        Update: {
          age_ticks?: number
          angle?: number
          ant_type_id?: number
          carried_resources?: Json | null
          colony_id?: string
          created_at?: string | null
          current_speed?: number
          energy?: number
          health?: number
          id?: string
          last_updated?: string | null
          mood?: string | null
          position_x?: number
          position_y?: number
          state?: string
          target_id?: string | null
          target_type?: string | null
          target_x?: number | null
          target_y?: number | null
          traits?: Json | null
        }
        Relationships: [
          {
            foreignKeyName: "ants_ant_type_id_fkey"
            columns: ["ant_type_id"]
            isOneToOne: false
            referencedRelation: "ant_types"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "ants_colony_id_fkey"
            columns: ["colony_id"]
            isOneToOne: false
            referencedRelation: "colonies"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "ants_colony_id_fkey"
            columns: ["colony_id"]
            isOneToOne: false
            referencedRelation: "colony_performance"
            referencedColumns: ["id"]
          },
        ]
      }
      climate_zones: {
        Row: {
          air_pressure: number | null
          center_x: number
          center_y: number
          created_at: string | null
          created_by: string | null
          elevation: number | null
          humidity: number
          id: string
          light_level: number
          radius: number
          seasonal_variations: Json | null
          simulation_id: string
          temperature: number
          vegetation_cover: number | null
          wind_direction: number | null
          wind_speed: number | null
          zone_name: string | null
        }
        Insert: {
          air_pressure?: number | null
          center_x: number
          center_y: number
          created_at?: string | null
          created_by?: string | null
          elevation?: number | null
          humidity: number
          id?: string
          light_level?: number
          radius?: number
          seasonal_variations?: Json | null
          simulation_id: string
          temperature: number
          vegetation_cover?: number | null
          wind_direction?: number | null
          wind_speed?: number | null
          zone_name?: string | null
        }
        Update: {
          air_pressure?: number | null
          center_x?: number
          center_y?: number
          created_at?: string | null
          created_by?: string | null
          elevation?: number | null
          humidity?: number
          id?: string
          light_level?: number
          radius?: number
          seasonal_variations?: Json | null
          simulation_id?: string
          temperature?: number
          vegetation_cover?: number | null
          wind_direction?: number | null
          wind_speed?: number | null
          zone_name?: string | null
        }
        Relationships: [
          {
            foreignKeyName: "climate_zones_simulation_id_fkey"
            columns: ["simulation_id"]
            isOneToOne: false
            referencedRelation: "ecosystem_health"
            referencedColumns: ["simulation_id"]
          },
          {
            foreignKeyName: "climate_zones_simulation_id_fkey"
            columns: ["simulation_id"]
            isOneToOne: false
            referencedRelation: "simulations"
            referencedColumns: ["id"]
          },
        ]
      }
      colonies: {
        Row: {
          aggression_level: number
          center_x: number
          center_y: number
          color_hue: number
          created_at: string | null
          id: string
          is_active: boolean | null
          name: string
          nest_level: number
          population: number
          radius: number
          resources: Json
          simulation_id: string
          territory_radius: number
        }
        Insert: {
          aggression_level?: number
          center_x: number
          center_y: number
          color_hue?: number
          created_at?: string | null
          id?: string
          is_active?: boolean | null
          name: string
          nest_level?: number
          population?: number
          radius?: number
          resources?: Json
          simulation_id: string
          territory_radius?: number
        }
        Update: {
          aggression_level?: number
          center_x?: number
          center_y?: number
          color_hue?: number
          created_at?: string | null
          id?: string
          is_active?: boolean | null
          name?: string
          nest_level?: number
          population?: number
          radius?: number
          resources?: Json
          simulation_id?: string
          territory_radius?: number
        }
        Relationships: [
          {
            foreignKeyName: "colonies_simulation_id_fkey"
            columns: ["simulation_id"]
            isOneToOne: false
            referencedRelation: "ecosystem_health"
            referencedColumns: ["simulation_id"]
          },
          {
            foreignKeyName: "colonies_simulation_id_fkey"
            columns: ["simulation_id"]
            isOneToOne: false
            referencedRelation: "simulations"
            referencedColumns: ["id"]
          },
        ]
      }
      colony_culture: {
        Row: {
          behavioral_modifiers: Json | null
          colony_id: string
          created_at: string | null
          cultural_trait: string
          developed_at_tick: number
          id: string
          influence_radius: number | null
          innovation_rate: number | null
          knowledge_traditions: Json | null
          origin_story: Json | null
          ritual_behaviors: Json | null
          trait_strength: number
        }
        Insert: {
          behavioral_modifiers?: Json | null
          colony_id: string
          created_at?: string | null
          cultural_trait: string
          developed_at_tick: number
          id?: string
          influence_radius?: number | null
          innovation_rate?: number | null
          knowledge_traditions?: Json | null
          origin_story?: Json | null
          ritual_behaviors?: Json | null
          trait_strength?: number
        }
        Update: {
          behavioral_modifiers?: Json | null
          colony_id?: string
          created_at?: string | null
          cultural_trait?: string
          developed_at_tick?: number
          id?: string
          influence_radius?: number | null
          innovation_rate?: number | null
          knowledge_traditions?: Json | null
          origin_story?: Json | null
          ritual_behaviors?: Json | null
          trait_strength?: number
        }
        Relationships: [
          {
            foreignKeyName: "colony_culture_colony_id_fkey"
            columns: ["colony_id"]
            isOneToOne: false
            referencedRelation: "colonies"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "colony_culture_colony_id_fkey"
            columns: ["colony_id"]
            isOneToOne: false
            referencedRelation: "colony_performance"
            referencedColumns: ["id"]
          },
        ]
      }
      colony_relations: {
        Row: {
          colony1_id: string
          colony2_id: string
          established_at: string | null
          expires_at: string | null
          id: string
          last_interaction_tick: number | null
          military_pacts: Json | null
          relationship_history: Json[] | null
          relationship_type: string
          territorial_agreements: Json | null
          trade_agreements: Json | null
          trust_level: number
        }
        Insert: {
          colony1_id: string
          colony2_id: string
          established_at?: string | null
          expires_at?: string | null
          id?: string
          last_interaction_tick?: number | null
          military_pacts?: Json | null
          relationship_history?: Json[] | null
          relationship_type: string
          territorial_agreements?: Json | null
          trade_agreements?: Json | null
          trust_level?: number
        }
        Update: {
          colony1_id?: string
          colony2_id?: string
          established_at?: string | null
          expires_at?: string | null
          id?: string
          last_interaction_tick?: number | null
          military_pacts?: Json | null
          relationship_history?: Json[] | null
          relationship_type?: string
          territorial_agreements?: Json | null
          trade_agreements?: Json | null
          trust_level?: number
        }
        Relationships: [
          {
            foreignKeyName: "colony_relations_colony1_id_fkey"
            columns: ["colony1_id"]
            isOneToOne: false
            referencedRelation: "colonies"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "colony_relations_colony1_id_fkey"
            columns: ["colony1_id"]
            isOneToOne: false
            referencedRelation: "colony_performance"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "colony_relations_colony2_id_fkey"
            columns: ["colony2_id"]
            isOneToOne: false
            referencedRelation: "colonies"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "colony_relations_colony2_id_fkey"
            columns: ["colony2_id"]
            isOneToOne: false
            referencedRelation: "colony_performance"
            referencedColumns: ["id"]
          },
        ]
      }
      colony_upgrades: {
        Row: {
          colony_id: string
          cost_paid: Json | null
          effects: Json | null
          id: string
          level: number
          unlocked_at: string | null
          upgrade_type: string
        }
        Insert: {
          colony_id: string
          cost_paid?: Json | null
          effects?: Json | null
          id?: string
          level?: number
          unlocked_at?: string | null
          upgrade_type: string
        }
        Update: {
          colony_id?: string
          cost_paid?: Json | null
          effects?: Json | null
          id?: string
          level?: number
          unlocked_at?: string | null
          upgrade_type?: string
        }
        Relationships: [
          {
            foreignKeyName: "colony_upgrades_colony_id_fkey"
            columns: ["colony_id"]
            isOneToOne: false
            referencedRelation: "colonies"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "colony_upgrades_colony_id_fkey"
            columns: ["colony_id"]
            isOneToOne: false
            referencedRelation: "colony_performance"
            referencedColumns: ["id"]
          },
        ]
      }
      decomposers: {
        Row: {
          created_at: string | null
          decomposer_type: string
          efficiency: number
          id: string
          nutrient_output: Json | null
          optimal_ph: number | null
          optimal_temperature: number | null
          population: number
          position_x: number
          position_y: number
          radius: number
          simulation_id: string
          target_material: string | null
        }
        Insert: {
          created_at?: string | null
          decomposer_type: string
          efficiency?: number
          id?: string
          nutrient_output?: Json | null
          optimal_ph?: number | null
          optimal_temperature?: number | null
          population?: number
          position_x: number
          position_y: number
          radius?: number
          simulation_id: string
          target_material?: string | null
        }
        Update: {
          created_at?: string | null
          decomposer_type?: string
          efficiency?: number
          id?: string
          nutrient_output?: Json | null
          optimal_ph?: number | null
          optimal_temperature?: number | null
          population?: number
          position_x?: number
          position_y?: number
          radius?: number
          simulation_id?: string
          target_material?: string | null
        }
        Relationships: [
          {
            foreignKeyName: "decomposers_simulation_id_fkey"
            columns: ["simulation_id"]
            isOneToOne: false
            referencedRelation: "ecosystem_health"
            referencedColumns: ["simulation_id"]
          },
          {
            foreignKeyName: "decomposers_simulation_id_fkey"
            columns: ["simulation_id"]
            isOneToOne: false
            referencedRelation: "simulations"
            referencedColumns: ["id"]
          },
        ]
      }
      diseases: {
        Row: {
          affected_species: string[] | null
          created_at: string | null
          disease_name: string
          environmental_survival: number | null
          id: string
          immunity_duration: number | null
          incubation_period: number
          mortality_rate: number
          mutation_rate: number | null
          pathogen_type: string
          recovery_rate: number
          simulation_id: string
          symptoms: Json | null
          transmission_method: string
          transmission_rate: number
        }
        Insert: {
          affected_species?: string[] | null
          created_at?: string | null
          disease_name: string
          environmental_survival?: number | null
          id?: string
          immunity_duration?: number | null
          incubation_period?: number
          mortality_rate?: number
          mutation_rate?: number | null
          pathogen_type: string
          recovery_rate?: number
          simulation_id: string
          symptoms?: Json | null
          transmission_method: string
          transmission_rate?: number
        }
        Update: {
          affected_species?: string[] | null
          created_at?: string | null
          disease_name?: string
          environmental_survival?: number | null
          id?: string
          immunity_duration?: number | null
          incubation_period?: number
          mortality_rate?: number
          mutation_rate?: number | null
          pathogen_type?: string
          recovery_rate?: number
          simulation_id?: string
          symptoms?: Json | null
          transmission_method?: string
          transmission_rate?: number
        }
        Relationships: [
          {
            foreignKeyName: "diseases_simulation_id_fkey"
            columns: ["simulation_id"]
            isOneToOne: false
            referencedRelation: "ecosystem_health"
            referencedColumns: ["simulation_id"]
          },
          {
            foreignKeyName: "diseases_simulation_id_fkey"
            columns: ["simulation_id"]
            isOneToOne: false
            referencedRelation: "simulations"
            referencedColumns: ["id"]
          },
        ]
      }
      espionage_missions: {
        Row: {
          completed_at_tick: number | null
          cover_identity: string | null
          created_at: string | null
          discovery_risk: number
          id: string
          intelligence_gathered: Json | null
          mission_status: string
          mission_type: string
          objectives: Json
          origin_colony_id: string
          resources_stolen: Json | null
          spy_ant_id: string
          started_at_tick: number | null
          success_rating: number | null
          target_colony_id: string
        }
        Insert: {
          completed_at_tick?: number | null
          cover_identity?: string | null
          created_at?: string | null
          discovery_risk?: number
          id?: string
          intelligence_gathered?: Json | null
          mission_status?: string
          mission_type: string
          objectives: Json
          origin_colony_id: string
          resources_stolen?: Json | null
          spy_ant_id: string
          started_at_tick?: number | null
          success_rating?: number | null
          target_colony_id: string
        }
        Update: {
          completed_at_tick?: number | null
          cover_identity?: string | null
          created_at?: string | null
          discovery_risk?: number
          id?: string
          intelligence_gathered?: Json | null
          mission_status?: string
          mission_type?: string
          objectives?: Json
          origin_colony_id?: string
          resources_stolen?: Json | null
          spy_ant_id?: string
          started_at_tick?: number | null
          success_rating?: number | null
          target_colony_id?: string
        }
        Relationships: [
          {
            foreignKeyName: "espionage_missions_origin_colony_id_fkey"
            columns: ["origin_colony_id"]
            isOneToOne: false
            referencedRelation: "colonies"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "espionage_missions_origin_colony_id_fkey"
            columns: ["origin_colony_id"]
            isOneToOne: false
            referencedRelation: "colony_performance"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "espionage_missions_spy_ant_id_fkey"
            columns: ["spy_ant_id"]
            isOneToOne: false
            referencedRelation: "ants"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "espionage_missions_target_colony_id_fkey"
            columns: ["target_colony_id"]
            isOneToOne: false
            referencedRelation: "colonies"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "espionage_missions_target_colony_id_fkey"
            columns: ["target_colony_id"]
            isOneToOne: false
            referencedRelation: "colony_performance"
            referencedColumns: ["id"]
          },
        ]
      }
      fire_zones: {
        Row: {
          casualties: Json | null
          center_x: number
          center_y: number
          created_at: string | null
          extinguished_at_tick: number | null
          fuel_remaining: number
          id: string
          ignition_source: string | null
          intensity: number
          radius: number
          simulation_id: string
          spread_rate: number
          started_at_tick: number
          suppression_efforts: Json | null
          wind_influence: number | null
        }
        Insert: {
          casualties?: Json | null
          center_x: number
          center_y: number
          created_at?: string | null
          extinguished_at_tick?: number | null
          fuel_remaining?: number
          id?: string
          ignition_source?: string | null
          intensity?: number
          radius?: number
          simulation_id: string
          spread_rate?: number
          started_at_tick: number
          suppression_efforts?: Json | null
          wind_influence?: number | null
        }
        Update: {
          casualties?: Json | null
          center_x?: number
          center_y?: number
          created_at?: string | null
          extinguished_at_tick?: number | null
          fuel_remaining?: number
          id?: string
          ignition_source?: string | null
          intensity?: number
          radius?: number
          simulation_id?: string
          spread_rate?: number
          started_at_tick?: number
          suppression_efforts?: Json | null
          wind_influence?: number | null
        }
        Relationships: [
          {
            foreignKeyName: "fire_zones_simulation_id_fkey"
            columns: ["simulation_id"]
            isOneToOne: false
            referencedRelation: "ecosystem_health"
            referencedColumns: ["simulation_id"]
          },
          {
            foreignKeyName: "fire_zones_simulation_id_fkey"
            columns: ["simulation_id"]
            isOneToOne: false
            referencedRelation: "simulations"
            referencedColumns: ["id"]
          },
        ]
      }
      food_sources: {
        Row: {
          amount: number
          created_at: string | null
          discovery_difficulty: number | null
          food_type: string
          id: string
          is_renewable: boolean | null
          max_amount: number
          nutritional_value: number
          position_x: number
          position_y: number
          regeneration_rate: number | null
          simulation_id: string
          spoilage_rate: number | null
        }
        Insert: {
          amount: number
          created_at?: string | null
          discovery_difficulty?: number | null
          food_type: string
          id?: string
          is_renewable?: boolean | null
          max_amount: number
          nutritional_value?: number
          position_x: number
          position_y: number
          regeneration_rate?: number | null
          simulation_id: string
          spoilage_rate?: number | null
        }
        Update: {
          amount?: number
          created_at?: string | null
          discovery_difficulty?: number | null
          food_type?: string
          id?: string
          is_renewable?: boolean | null
          max_amount?: number
          nutritional_value?: number
          position_x?: number
          position_y?: number
          regeneration_rate?: number | null
          simulation_id?: string
          spoilage_rate?: number | null
        }
        Relationships: [
          {
            foreignKeyName: "food_sources_simulation_id_fkey"
            columns: ["simulation_id"]
            isOneToOne: false
            referencedRelation: "ecosystem_health"
            referencedColumns: ["simulation_id"]
          },
          {
            foreignKeyName: "food_sources_simulation_id_fkey"
            columns: ["simulation_id"]
            isOneToOne: false
            referencedRelation: "simulations"
            referencedColumns: ["id"]
          },
        ]
      }
      infections: {
        Row: {
          created_at: string | null
          disease_id: string
          host_id: string
          host_type: string
          id: string
          infected_at_tick: number
          infection_stage: string
          recovery_tick: number | null
          strain_mutations: Json | null
          symptoms_start_tick: number | null
          transmission_events: number | null
        }
        Insert: {
          created_at?: string | null
          disease_id: string
          host_id: string
          host_type: string
          id?: string
          infected_at_tick: number
          infection_stage: string
          recovery_tick?: number | null
          strain_mutations?: Json | null
          symptoms_start_tick?: number | null
          transmission_events?: number | null
        }
        Update: {
          created_at?: string | null
          disease_id?: string
          host_id?: string
          host_type?: string
          id?: string
          infected_at_tick?: number
          infection_stage?: string
          recovery_tick?: number | null
          strain_mutations?: Json | null
          symptoms_start_tick?: number | null
          transmission_events?: number | null
        }
        Relationships: [
          {
            foreignKeyName: "infections_disease_id_fkey"
            columns: ["disease_id"]
            isOneToOne: false
            referencedRelation: "diseases"
            referencedColumns: ["id"]
          },
        ]
      }
      migration_patterns: {
        Row: {
          colony_id: string
          created_at: string | null
          destination_preferences: Json | null
          id: string
          last_migration_tick: number | null
          migration_routes: Json[] | null
          migration_speed: number | null
          pattern_type: string
          preparation_time: number | null
          seasonal_schedule: Json | null
          survival_rate: number | null
          trigger_conditions: Json
        }
        Insert: {
          colony_id: string
          created_at?: string | null
          destination_preferences?: Json | null
          id?: string
          last_migration_tick?: number | null
          migration_routes?: Json[] | null
          migration_speed?: number | null
          pattern_type: string
          preparation_time?: number | null
          seasonal_schedule?: Json | null
          survival_rate?: number | null
          trigger_conditions: Json
        }
        Update: {
          colony_id?: string
          created_at?: string | null
          destination_preferences?: Json | null
          id?: string
          last_migration_tick?: number | null
          migration_routes?: Json[] | null
          migration_speed?: number | null
          pattern_type?: string
          preparation_time?: number | null
          seasonal_schedule?: Json | null
          survival_rate?: number | null
          trigger_conditions?: Json
        }
        Relationships: [
          {
            foreignKeyName: "migration_patterns_colony_id_fkey"
            columns: ["colony_id"]
            isOneToOne: false
            referencedRelation: "colonies"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "migration_patterns_colony_id_fkey"
            columns: ["colony_id"]
            isOneToOne: false
            referencedRelation: "colony_performance"
            referencedColumns: ["id"]
          },
        ]
      }
      obstacles: {
        Row: {
          affects_pheromones: boolean | null
          height: number | null
          id: string
          is_passable: boolean | null
          movement_cost: number | null
          obstacle_type: string
          polygon_points: Json | null
          position_x: number
          position_y: number
          radius: number | null
          shape: string
          simulation_id: string
          visual_properties: Json | null
          width: number | null
        }
        Insert: {
          affects_pheromones?: boolean | null
          height?: number | null
          id?: string
          is_passable?: boolean | null
          movement_cost?: number | null
          obstacle_type: string
          polygon_points?: Json | null
          position_x: number
          position_y: number
          radius?: number | null
          shape: string
          simulation_id: string
          visual_properties?: Json | null
          width?: number | null
        }
        Update: {
          affects_pheromones?: boolean | null
          height?: number | null
          id?: string
          is_passable?: boolean | null
          movement_cost?: number | null
          obstacle_type?: string
          polygon_points?: Json | null
          position_x?: number
          position_y?: number
          radius?: number | null
          shape?: string
          simulation_id?: string
          visual_properties?: Json | null
          width?: number | null
        }
        Relationships: [
          {
            foreignKeyName: "obstacles_simulation_id_fkey"
            columns: ["simulation_id"]
            isOneToOne: false
            referencedRelation: "ecosystem_health"
            referencedColumns: ["simulation_id"]
          },
          {
            foreignKeyName: "obstacles_simulation_id_fkey"
            columns: ["simulation_id"]
            isOneToOne: false
            referencedRelation: "simulations"
            referencedColumns: ["id"]
          },
        ]
      }
      pheromone_trails: {
        Row: {
          colony_id: string
          created_at: string | null
          decay_rate: number
          expires_at: string | null
          id: string
          position_x: number
          position_y: number
          source_ant_id: string | null
          strength: number
          target_food_id: string | null
          trail_type: string
        }
        Insert: {
          colony_id: string
          created_at?: string | null
          decay_rate?: number
          expires_at?: string | null
          id?: string
          position_x: number
          position_y: number
          source_ant_id?: string | null
          strength: number
          target_food_id?: string | null
          trail_type: string
        }
        Update: {
          colony_id?: string
          created_at?: string | null
          decay_rate?: number
          expires_at?: string | null
          id?: string
          position_x?: number
          position_y?: number
          source_ant_id?: string | null
          strength?: number
          target_food_id?: string | null
          trail_type?: string
        }
        Relationships: [
          {
            foreignKeyName: "pheromone_trails_colony_id_fkey"
            columns: ["colony_id"]
            isOneToOne: false
            referencedRelation: "colonies"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "pheromone_trails_colony_id_fkey"
            columns: ["colony_id"]
            isOneToOne: false
            referencedRelation: "colony_performance"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "pheromone_trails_source_ant_id_fkey"
            columns: ["source_ant_id"]
            isOneToOne: false
            referencedRelation: "ants"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "pheromone_trails_target_food_id_fkey"
            columns: ["target_food_id"]
            isOneToOne: false
            referencedRelation: "food_sources"
            referencedColumns: ["id"]
          },
        ]
      }
      plants: {
        Row: {
          age_ticks: number
          canopy_radius: number
          created_at: string | null
          fruit_production_rate: number | null
          growth_rate: number
          health: number
          id: string
          max_size: number
          nutrient_requirements: Json | null
          oxygen_production: number | null
          plant_type: string
          position_x: number
          position_y: number
          root_radius: number
          seasonal_behavior: Json | null
          simulation_id: string
          size: number
          species: string
          symbiotic_species: string[] | null
          water_requirement: number
        }
        Insert: {
          age_ticks?: number
          canopy_radius?: number
          created_at?: string | null
          fruit_production_rate?: number | null
          growth_rate?: number
          health?: number
          id?: string
          max_size?: number
          nutrient_requirements?: Json | null
          oxygen_production?: number | null
          plant_type: string
          position_x: number
          position_y: number
          root_radius?: number
          seasonal_behavior?: Json | null
          simulation_id: string
          size?: number
          species: string
          symbiotic_species?: string[] | null
          water_requirement?: number
        }
        Update: {
          age_ticks?: number
          canopy_radius?: number
          created_at?: string | null
          fruit_production_rate?: number | null
          growth_rate?: number
          health?: number
          id?: string
          max_size?: number
          nutrient_requirements?: Json | null
          oxygen_production?: number | null
          plant_type?: string
          position_x?: number
          position_y?: number
          root_radius?: number
          seasonal_behavior?: Json | null
          simulation_id?: string
          size?: number
          species?: string
          symbiotic_species?: string[] | null
          water_requirement?: number
        }
        Relationships: [
          {
            foreignKeyName: "plants_simulation_id_fkey"
            columns: ["simulation_id"]
            isOneToOne: false
            referencedRelation: "ecosystem_health"
            referencedColumns: ["simulation_id"]
          },
          {
            foreignKeyName: "plants_simulation_id_fkey"
            columns: ["simulation_id"]
            isOneToOne: false
            referencedRelation: "simulations"
            referencedColumns: ["id"]
          },
        ]
      }
      predators: {
        Row: {
          angle: number
          attack_radius: number
          detection_radius: number
          health: number
          hunger: number
          id: string
          last_hunt_tick: number | null
          position_x: number
          position_y: number
          predator_type: string
          simulation_id: string
          speed: number
          state: string | null
          target_ant_id: string | null
          territory_center_x: number | null
          territory_center_y: number | null
          territory_radius: number | null
        }
        Insert: {
          angle?: number
          attack_radius?: number
          detection_radius?: number
          health?: number
          hunger?: number
          id?: string
          last_hunt_tick?: number | null
          position_x: number
          position_y: number
          predator_type: string
          simulation_id: string
          speed?: number
          state?: string | null
          target_ant_id?: string | null
          territory_center_x?: number | null
          territory_center_y?: number | null
          territory_radius?: number | null
        }
        Update: {
          angle?: number
          attack_radius?: number
          detection_radius?: number
          health?: number
          hunger?: number
          id?: string
          last_hunt_tick?: number | null
          position_x?: number
          position_y?: number
          predator_type?: string
          simulation_id?: string
          speed?: number
          state?: string | null
          target_ant_id?: string | null
          territory_center_x?: number | null
          territory_center_y?: number | null
          territory_radius?: number | null
        }
        Relationships: [
          {
            foreignKeyName: "predators_simulation_id_fkey"
            columns: ["simulation_id"]
            isOneToOne: false
            referencedRelation: "ecosystem_health"
            referencedColumns: ["simulation_id"]
          },
          {
            foreignKeyName: "predators_simulation_id_fkey"
            columns: ["simulation_id"]
            isOneToOne: false
            referencedRelation: "simulations"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "predators_target_ant_id_fkey"
            columns: ["target_ant_id"]
            isOneToOne: false
            referencedRelation: "ants"
            referencedColumns: ["id"]
          },
        ]
      }
      scenario_templates: {
        Row: {
          average_rating: number | null
          created_at: string | null
          creator_id: string | null
          description: string | null
          difficulty_rating: number | null
          id: string
          is_public: boolean | null
          name: string
          play_count: number | null
          tags: string[] | null
          world_config: Json
        }
        Insert: {
          average_rating?: number | null
          created_at?: string | null
          creator_id?: string | null
          description?: string | null
          difficulty_rating?: number | null
          id?: string
          is_public?: boolean | null
          name: string
          play_count?: number | null
          tags?: string[] | null
          world_config: Json
        }
        Update: {
          average_rating?: number | null
          created_at?: string | null
          creator_id?: string | null
          description?: string | null
          difficulty_rating?: number | null
          id?: string
          is_public?: boolean | null
          name?: string
          play_count?: number | null
          tags?: string[] | null
          world_config?: Json
        }
        Relationships: []
      }
      simulation_events: {
        Row: {
          center_x: number | null
          center_y: number | null
          created_at: string | null
          duration_ticks: number | null
          effects: Json | null
          event_type: string
          id: string
          is_active: boolean | null
          radius: number | null
          severity: number
          simulation_id: string
          start_tick: number
        }
        Insert: {
          center_x?: number | null
          center_y?: number | null
          created_at?: string | null
          duration_ticks?: number | null
          effects?: Json | null
          event_type: string
          id?: string
          is_active?: boolean | null
          radius?: number | null
          severity?: number
          simulation_id: string
          start_tick: number
        }
        Update: {
          center_x?: number | null
          center_y?: number | null
          created_at?: string | null
          duration_ticks?: number | null
          effects?: Json | null
          event_type?: string
          id?: string
          is_active?: boolean | null
          radius?: number | null
          severity?: number
          simulation_id?: string
          start_tick?: number
        }
        Relationships: [
          {
            foreignKeyName: "simulation_events_simulation_id_fkey"
            columns: ["simulation_id"]
            isOneToOne: false
            referencedRelation: "ecosystem_health"
            referencedColumns: ["simulation_id"]
          },
          {
            foreignKeyName: "simulation_events_simulation_id_fkey"
            columns: ["simulation_id"]
            isOneToOne: false
            referencedRelation: "simulations"
            referencedColumns: ["id"]
          },
        ]
      }
      simulation_stats: {
        Row: {
          active_combats: number
          average_ant_health: number | null
          dominant_colony_id: string | null
          id: string
          pheromone_trail_count: number
          recorded_at: string | null
          simulation_id: string
          tick_number: number
          total_ants: number
          total_distance_traveled: number
          total_food_collected: number
          weather_effects_active: number
        }
        Insert: {
          active_combats?: number
          average_ant_health?: number | null
          dominant_colony_id?: string | null
          id?: string
          pheromone_trail_count?: number
          recorded_at?: string | null
          simulation_id: string
          tick_number: number
          total_ants: number
          total_distance_traveled?: number
          total_food_collected?: number
          weather_effects_active?: number
        }
        Update: {
          active_combats?: number
          average_ant_health?: number | null
          dominant_colony_id?: string | null
          id?: string
          pheromone_trail_count?: number
          recorded_at?: string | null
          simulation_id?: string
          tick_number?: number
          total_ants?: number
          total_distance_traveled?: number
          total_food_collected?: number
          weather_effects_active?: number
        }
        Relationships: [
          {
            foreignKeyName: "simulation_stats_dominant_colony_id_fkey"
            columns: ["dominant_colony_id"]
            isOneToOne: false
            referencedRelation: "colonies"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "simulation_stats_dominant_colony_id_fkey"
            columns: ["dominant_colony_id"]
            isOneToOne: false
            referencedRelation: "colony_performance"
            referencedColumns: ["id"]
          },
          {
            foreignKeyName: "simulation_stats_simulation_id_fkey"
            columns: ["simulation_id"]
            isOneToOne: false
            referencedRelation: "ecosystem_health"
            referencedColumns: ["simulation_id"]
          },
          {
            foreignKeyName: "simulation_stats_simulation_id_fkey"
            columns: ["simulation_id"]
            isOneToOne: false
            referencedRelation: "simulations"
            referencedColumns: ["id"]
          },
        ]
      }
      simulations: {
        Row: {
          created_at: string | null
          current_tick: number | null
          description: string | null
          id: string
          is_active: boolean | null
          name: string
          season: string | null
          simulation_speed: number | null
          time_of_day: number | null
          updated_at: string | null
          weather_intensity: number | null
          weather_type: string | null
          world_height: number
          world_width: number
        }
        Insert: {
          created_at?: string | null
          current_tick?: number | null
          description?: string | null
          id?: string
          is_active?: boolean | null
          name: string
          season?: string | null
          simulation_speed?: number | null
          time_of_day?: number | null
          updated_at?: string | null
          weather_intensity?: number | null
          weather_type?: string | null
          world_height?: number
          world_width?: number
        }
        Update: {
          created_at?: string | null
          current_tick?: number | null
          description?: string | null
          id?: string
          is_active?: boolean | null
          name?: string
          season?: string | null
          simulation_speed?: number | null
          time_of_day?: number | null
          updated_at?: string | null
          weather_intensity?: number | null
          weather_type?: string | null
          world_height?: number
          world_width?: number
        }
        Relationships: []
      }
      soil_zones: {
        Row: {
          center_x: number
          center_y: number
          compaction: number
          contamination_level: number | null
          created_at: string | null
          drainage_rate: number | null
          fertility_score: number | null
          id: string
          last_updated_tick: number | null
          microbial_activity: number | null
          moisture_content: number
          nutrients: Json
          ph_level: number
          radius: number
          simulation_id: string
          soil_type: string
          temperature: number | null
          zone_name: string | null
        }
        Insert: {
          center_x: number
          center_y: number
          compaction?: number
          contamination_level?: number | null
          created_at?: string | null
          drainage_rate?: number | null
          fertility_score?: number | null
          id?: string
          last_updated_tick?: number | null
          microbial_activity?: number | null
          moisture_content?: number
          nutrients: Json
          ph_level?: number
          radius?: number
          simulation_id: string
          soil_type: string
          temperature?: number | null
          zone_name?: string | null
        }
        Update: {
          center_x?: number
          center_y?: number
          compaction?: number
          contamination_level?: number | null
          created_at?: string | null
          drainage_rate?: number | null
          fertility_score?: number | null
          id?: string
          last_updated_tick?: number | null
          microbial_activity?: number | null
          moisture_content?: number
          nutrients?: Json
          ph_level?: number
          radius?: number
          simulation_id?: string
          soil_type?: string
          temperature?: number | null
          zone_name?: string | null
        }
        Relationships: [
          {
            foreignKeyName: "soil_zones_simulation_id_fkey"
            columns: ["simulation_id"]
            isOneToOne: false
            referencedRelation: "ecosystem_health"
            referencedColumns: ["simulation_id"]
          },
          {
            foreignKeyName: "soil_zones_simulation_id_fkey"
            columns: ["simulation_id"]
            isOneToOne: false
            referencedRelation: "simulations"
            referencedColumns: ["id"]
          },
        ]
      }
      species: {
        Row: {
          created_at: string | null
          diet_type: string
          environmental_preferences: Json | null
          food_requirements: Json | null
          id: string
          mobility: string
          mortality_rate: number | null
          population: number
          position_x: number
          position_y: number
          reproduction_rate: number | null
          simulation_id: string
          species_name: string
          species_type: string
          symbiotic_relationships: Json | null
          territory_radius: number | null
        }
        Insert: {
          created_at?: string | null
          diet_type: string
          environmental_preferences?: Json | null
          food_requirements?: Json | null
          id?: string
          mobility: string
          mortality_rate?: number | null
          population?: number
          position_x: number
          position_y: number
          reproduction_rate?: number | null
          simulation_id: string
          species_name: string
          species_type: string
          symbiotic_relationships?: Json | null
          territory_radius?: number | null
        }
        Update: {
          created_at?: string | null
          diet_type?: string
          environmental_preferences?: Json | null
          food_requirements?: Json | null
          id?: string
          mobility?: string
          mortality_rate?: number | null
          population?: number
          position_x?: number
          position_y?: number
          reproduction_rate?: number | null
          simulation_id?: string
          species_name?: string
          species_type?: string
          symbiotic_relationships?: Json | null
          territory_radius?: number | null
        }
        Relationships: [
          {
            foreignKeyName: "species_simulation_id_fkey"
            columns: ["simulation_id"]
            isOneToOne: false
            referencedRelation: "ecosystem_health"
            referencedColumns: ["simulation_id"]
          },
          {
            foreignKeyName: "species_simulation_id_fkey"
            columns: ["simulation_id"]
            isOneToOne: false
            referencedRelation: "simulations"
            referencedColumns: ["id"]
          },
        ]
      }
      water_bodies: {
        Row: {
          center_x: number
          center_y: number
          created_at: string | null
          depth: number
          evaporation_rate: number | null
          flow_direction: number | null
          flow_speed: number | null
          id: string
          is_seasonal: boolean | null
          length: number | null
          polygon_points: Json | null
          radius: number | null
          shape: string
          simulation_id: string
          temperature: number | null
          water_quality: number | null
          water_type: string
          width: number | null
        }
        Insert: {
          center_x: number
          center_y: number
          created_at?: string | null
          depth?: number
          evaporation_rate?: number | null
          flow_direction?: number | null
          flow_speed?: number | null
          id?: string
          is_seasonal?: boolean | null
          length?: number | null
          polygon_points?: Json | null
          radius?: number | null
          shape: string
          simulation_id: string
          temperature?: number | null
          water_quality?: number | null
          water_type: string
          width?: number | null
        }
        Update: {
          center_x?: number
          center_y?: number
          created_at?: string | null
          depth?: number
          evaporation_rate?: number | null
          flow_direction?: number | null
          flow_speed?: number | null
          id?: string
          is_seasonal?: boolean | null
          length?: number | null
          polygon_points?: Json | null
          radius?: number | null
          shape?: string
          simulation_id?: string
          temperature?: number | null
          water_quality?: number | null
          water_type?: string
          width?: number | null
        }
        Relationships: [
          {
            foreignKeyName: "water_bodies_simulation_id_fkey"
            columns: ["simulation_id"]
            isOneToOne: false
            referencedRelation: "ecosystem_health"
            referencedColumns: ["simulation_id"]
          },
          {
            foreignKeyName: "water_bodies_simulation_id_fkey"
            columns: ["simulation_id"]
            isOneToOne: false
            referencedRelation: "simulations"
            referencedColumns: ["id"]
          },
        ]
      }
      weather_systems: {
        Row: {
          center_x: number | null
          center_y: number | null
          created_at: string | null
          duration_remaining: number
          effects: Json
          forecast_accuracy: number | null
          id: string
          intensity: number
          movement_vector_x: number | null
          movement_vector_y: number | null
          pressure_change: number | null
          radius: number | null
          simulation_id: string
          started_at_tick: number
          weather_type: string
        }
        Insert: {
          center_x?: number | null
          center_y?: number | null
          created_at?: string | null
          duration_remaining: number
          effects: Json
          forecast_accuracy?: number | null
          id?: string
          intensity?: number
          movement_vector_x?: number | null
          movement_vector_y?: number | null
          pressure_change?: number | null
          radius?: number | null
          simulation_id: string
          started_at_tick: number
          weather_type: string
        }
        Update: {
          center_x?: number | null
          center_y?: number | null
          created_at?: string | null
          duration_remaining?: number
          effects?: Json
          forecast_accuracy?: number | null
          id?: string
          intensity?: number
          movement_vector_x?: number | null
          movement_vector_y?: number | null
          pressure_change?: number | null
          radius?: number | null
          simulation_id?: string
          started_at_tick?: number
          weather_type?: string
        }
        Relationships: [
          {
            foreignKeyName: "weather_systems_simulation_id_fkey"
            columns: ["simulation_id"]
            isOneToOne: false
            referencedRelation: "ecosystem_health"
            referencedColumns: ["simulation_id"]
          },
          {
            foreignKeyName: "weather_systems_simulation_id_fkey"
            columns: ["simulation_id"]
            isOneToOne: false
            referencedRelation: "simulations"
            referencedColumns: ["id"]
          },
        ]
      }
    }
    Views: {
      colony_performance: {
        Row: {
          active_pheromone_trails: number | null
          ants_with_food: number | null
          avg_ant_health: number | null
          id: string | null
          name: string | null
          nearby_food_sources: number | null
          population: number | null
          resources: Json | null
        }
        Relationships: []
      }
      ecosystem_health: {
        Row: {
          active_colonies: number | null
          active_diseases: number | null
          active_fires: number | null
          avg_humidity: number | null
          avg_soil_fertility: number | null
          avg_temperature: number | null
          name: string | null
          other_species_count: number | null
          plant_count: number | null
          simulation_id: string | null
          total_ants: number | null
          water_sources: number | null
        }
        Relationships: []
      }
    }
    Functions: {
      [_ in never]: never
    }
    Enums: {
      [_ in never]: never
    }
    CompositeTypes: {
      [_ in never]: never
    }
  }
}

type DefaultSchema = Database[Extract<keyof Database, "public">]

export type Tables<
  DefaultSchemaTableNameOrOptions extends
    | keyof (DefaultSchema["Tables"] & DefaultSchema["Views"])
    | { schema: keyof Database },
  TableName extends DefaultSchemaTableNameOrOptions extends {
    schema: keyof Database
  }
    ? keyof (Database[DefaultSchemaTableNameOrOptions["schema"]]["Tables"] &
        Database[DefaultSchemaTableNameOrOptions["schema"]]["Views"])
    : never = never,
> = DefaultSchemaTableNameOrOptions extends { schema: keyof Database }
  ? (Database[DefaultSchemaTableNameOrOptions["schema"]]["Tables"] &
      Database[DefaultSchemaTableNameOrOptions["schema"]]["Views"])[TableName] extends {
      Row: infer R
    }
    ? R
    : never
  : DefaultSchemaTableNameOrOptions extends keyof (DefaultSchema["Tables"] &
        DefaultSchema["Views"])
    ? (DefaultSchema["Tables"] &
        DefaultSchema["Views"])[DefaultSchemaTableNameOrOptions] extends {
        Row: infer R
      }
      ? R
      : never
    : never

export type TablesInsert<
  DefaultSchemaTableNameOrOptions extends
    | keyof DefaultSchema["Tables"]
    | { schema: keyof Database },
  TableName extends DefaultSchemaTableNameOrOptions extends {
    schema: keyof Database
  }
    ? keyof Database[DefaultSchemaTableNameOrOptions["schema"]]["Tables"]
    : never = never,
> = DefaultSchemaTableNameOrOptions extends { schema: keyof Database }
  ? Database[DefaultSchemaTableNameOrOptions["schema"]]["Tables"][TableName] extends {
      Insert: infer I
    }
    ? I
    : never
  : DefaultSchemaTableNameOrOptions extends keyof DefaultSchema["Tables"]
    ? DefaultSchema["Tables"][DefaultSchemaTableNameOrOptions] extends {
        Insert: infer I
      }
      ? I
      : never
    : never

export type TablesUpdate<
  DefaultSchemaTableNameOrOptions extends
    | keyof DefaultSchema["Tables"]
    | { schema: keyof Database },
  TableName extends DefaultSchemaTableNameOrOptions extends {
    schema: keyof Database
  }
    ? keyof Database[DefaultSchemaTableNameOrOptions["schema"]]["Tables"]
    : never = never,
> = DefaultSchemaTableNameOrOptions extends { schema: keyof Database }
  ? Database[DefaultSchemaTableNameOrOptions["schema"]]["Tables"][TableName] extends {
      Update: infer U
    }
    ? U
    : never
  : DefaultSchemaTableNameOrOptions extends keyof DefaultSchema["Tables"]
    ? DefaultSchema["Tables"][DefaultSchemaTableNameOrOptions] extends {
        Update: infer U
      }
      ? U
      : never
    : never

export type Enums<
  DefaultSchemaEnumNameOrOptions extends
    | keyof DefaultSchema["Enums"]
    | { schema: keyof Database },
  EnumName extends DefaultSchemaEnumNameOrOptions extends {
    schema: keyof Database
  }
    ? keyof Database[DefaultSchemaEnumNameOrOptions["schema"]]["Enums"]
    : never = never,
> = DefaultSchemaEnumNameOrOptions extends { schema: keyof Database }
  ? Database[DefaultSchemaEnumNameOrOptions["schema"]]["Enums"][EnumName]
  : DefaultSchemaEnumNameOrOptions extends keyof DefaultSchema["Enums"]
    ? DefaultSchema["Enums"][DefaultSchemaEnumNameOrOptions]
    : never

export type CompositeTypes<
  PublicCompositeTypeNameOrOptions extends
    | keyof DefaultSchema["CompositeTypes"]
    | { schema: keyof Database },
  CompositeTypeName extends PublicCompositeTypeNameOrOptions extends {
    schema: keyof Database
  }
    ? keyof Database[PublicCompositeTypeNameOrOptions["schema"]]["CompositeTypes"]
    : never = never,
> = PublicCompositeTypeNameOrOptions extends { schema: keyof Database }
  ? Database[PublicCompositeTypeNameOrOptions["schema"]]["CompositeTypes"][CompositeTypeName]
  : PublicCompositeTypeNameOrOptions extends keyof DefaultSchema["CompositeTypes"]
    ? DefaultSchema["CompositeTypes"][PublicCompositeTypeNameOrOptions]
    : never

export const Constants = {
  graphql_public: {
    Enums: {},
  },
  public: {
    Enums: {},
  },
} as const

