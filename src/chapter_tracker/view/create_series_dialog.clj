(ns chapter-tracker.view.create-series-dialog
  (:use chapter-tracker.view.tools)
  (:use chapter-tracker.model)
)
(import
  '(java.awt GridBagLayout GridBagConstraints Dimension)
  '(javax.swing JFrame JLabel JTextField JComboBox)
)

(defn create-create-series-dialog[]
  (create-frame {:title "Create Series"}
                (let [media-type-field    (JComboBox. (to-array (fetch-media-records)))
                      series-name-field   (JTextField. 10)
                      ;series-dir-field    (JTextField. 10)
                     ]
                  (.setLayout frame (GridBagLayout.))

                  (add-with-constraints (JLabel. "Media Type:") (gridx 0) (gridy 0))
                  (add-with-constraints media-type-field (gridx 1) (gridy 0) (gridwidth 2) (fill GridBagConstraints/BOTH))

                  (add-with-constraints (JLabel. "Series Name:") (gridx 0) (gridy 1))
                  (add-with-constraints series-name-field (gridx 1) (gridy 1) (gridwidth 2) (fill GridBagConstraints/BOTH))

                  ;(add-with-constraints (JLabel. "Directory") (gridx 0) (gridy 2))
                  ;(add-with-constraints series-dir-field   (gridx 1) (gridy 2) (fill GridBagConstraints/BOTH))
                  ;(add-with-constraints (action-button "..."
                                                       ;(if-let [dir (choose-dir (-> media-type-field .getSelectedItem :base-dir))]
                                                         ;(.setText series-dir-field dir))
                                        ;) (gridx 2) (gridy 2) (fill GridBagConstraints/BOTH))

                  (add-with-constraints (action-button "SAVE"
                                                             (create-series (.getSelectedItem media-type-field)
                                                                            (.getText series-name-field))
                                                             (.dispose frame)
                                              )
                                        (gridx 0) (gridy 2) (gridwidth 3) (fill GridBagConstraints/BOTH))
                )
  )
)

