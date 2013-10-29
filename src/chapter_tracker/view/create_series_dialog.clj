(ns chapter-tracker.view.create-series-dialog (:gen-class)
  (:use chapter-tracker.view.tools)
  (:use chapter-tracker.model)
)
(import
  '(java.awt GridBagLayout GridBagConstraints Dimension)
  '(javax.swing JFrame JLabel JTextField JComboBox JCheckBox)
)

(defn create-create-series-dialog [update-serieses-list-function & [edit-series-id]]
  (create-frame {:title (if edit-series-id "Edit Series" "Create Series")}
                (let [media-type-field    (JComboBox. (to-array (fetch-media-records)))
                      series-name-field   (JTextField. 10)
                      episode-numbers-repeat-each-volume-field   (JCheckBox.)
                     ]
                  (.setLayout frame (GridBagLayout.))

                  (add-with-constraints (JLabel. "Media Type:") (gridx 0) (gridy 0))
                  (add-with-constraints media-type-field (gridx 1) (gridy 0) (gridwidth 2) (fill GridBagConstraints/BOTH))

                  (add-with-constraints (JLabel. "Series Name:") (gridx 0) (gridy 1))
                  (add-with-constraints series-name-field (gridx 1) (gridy 1) (gridwidth 2) (fill GridBagConstraints/BOTH))

                  (add-with-constraints (JLabel. "Episode Numbers Repeat Each Volume") (gridx 0) (gridy 2) (gridwidth 2))
                  (add-with-constraints episode-numbers-repeat-each-volume-field (gridx 2) (gridy 2) (fill GridBagConstraints/BOTH))

                  (when edit-series-id
                    (let [series (fetch-series-record edit-series-id)]
                      (.setSelectedItem media-type-field (->> (range (.getItemCount media-type-field))
                                                             (map #(.getItemAt media-type-field %))
                                                             (filter #(= % (:media series)))
                                                             first))
                      (.setText series-name-field (:series-name series))
                      (.setSelected episode-numbers-repeat-each-volume-field (not= 0 (or (:episode-numbers-repeat-each-volume series) 0)))
                    ))

                  (add-with-constraints (action-button "SAVE"
                                                       (if edit-series-id
                                                         ;when saving existing series:
                                                         (update-series edit-series-id {
                                                                                        :media_type (-> media-type-field .getSelectedItem :media-id)
                                                                                        :name (.getText series-name-field)
                                                                                        :numbers_repeat_each_volume (.isSelected episode-numbers-repeat-each-volume-field)
                                                                                       })
                                                         ;when creating new series:
                                                         (create-series (.getSelectedItem media-type-field)
                                                                        (.getText series-name-field)
                                                                        (.isSelected episode-numbers-repeat-each-volume-field)))
                                                       (update-serieses-list-function)
                                                       (.dispose frame)
                                        )
                                        (gridx 0) (gridy 3) (gridwidth 3) (fill GridBagConstraints/BOTH))
                )
  )
)

